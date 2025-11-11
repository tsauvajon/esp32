#[cfg(feature = "access-point")]
use core::{net::Ipv4Addr, str::FromStr};

use embassy_executor::Spawner;
#[cfg(feature = "access-point")]
use embassy_net::{Ipv4Cidr, StaticConfigV4};
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::rng::Rng;
#[cfg(feature = "access-point")]
use esp_radio::wifi::{AccessPointConfig, WifiApState};
use esp_radio::wifi::{AuthMethod, Interfaces, ModeConfig, WifiController, WifiDevice, WifiEvent};
#[cfg(feature = "station")]
use esp_radio::wifi::{ClientConfig, WifiStaState};
use log::{error, info};

use crate::mk_static;

// When in station mode: remote Wi-Fi credentials
// When in access point mode: credentials for clients to use
const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

pub async fn start_wifi(
    wifi_controller: WifiController<'static>,
    interfaces: Interfaces<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    // TODO: here and everywhere else, handle both station + AP being enabled at the same time
    #[cfg(feature = "station")]
    let wifi_interface = interfaces.sta;
    #[cfg(feature = "access-point")]
    let wifi_interface = interfaces.ap;
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    #[cfg(feature = "station")]
    let net_config = embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default());

    #[cfg(feature = "access-point")]
    let net_config = {
        const STATIC_IP: &str = "192.168.2.1/24";
        const GATEWAY_IP: &str = "192.168.2.1";

        let address = Ipv4Cidr::from_str(STATIC_IP).unwrap();
        let gateway = Some(Ipv4Addr::from_str(GATEWAY_IP).unwrap());
        embassy_net::Config::ipv4_static(StaticConfigV4 {
            address,
            gateway,
            dns_servers: Default::default(),
        })
    };

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(connection_task(wifi_controller)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();
    #[cfg(feature = "access-point")]
    spawner.spawn(dhcp_task(stack, gw_ip_addr_str)).unwrap();

    wait_for_connection(stack).await;

    stack
}

async fn wait_for_connection(stack: Stack<'_>) {
    let mut delay = Delay {};
    info!("Waiting for link to be up");
    while !stack.is_link_up() {
        delay.delay_ms(500).await;
    }

    #[cfg(feature = "station")]
    {
        info!("Waiting to get IP address");
        loop {
            if let Some(config) = stack.config_v4() {
                info!("Got IP: {}", config.address);
                break;
            }
            delay.delay_ms(100).await;
        }
    }

    #[cfg(feature = "access-point")]
    {
        info!("Waiting for a device to connect and browse http://{STATIC_IP}");
        while !stack.is_config_up() {
            delay.delay_ms(100).await;
        }

        stack
            .config_v4()
            .inspect(|config| info!("IPv4 config: {config:?}"));
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
#[cfg_attr(feature = "station", allow(unused_mut))]
async fn connection_task(mut controller: WifiController<'static>) {
    info!(
        "Starting connection. Device capabilities: {:?}",
        controller.capabilities()
    );
    let mut delay = Delay {};

    // TODO: join the two loops together, reusing common code
    #[cfg(feature = "station")]
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                delay.delay_ms(5_000).await
            }

            WifiStaState::Started
            | WifiStaState::Disconnected
            | WifiStaState::Stopped
            | WifiStaState::Invalid
            | _ => {
                info!("Wi-Fi Station status: {:?}", esp_radio::wifi::ap_state());
            }
        }

        if !controller.is_started().unwrap_or_default() {
            let ssid = SSID.try_into().unwrap();
            let pw = PASSWORD.try_into().unwrap();
            let client_config = ClientConfig::default()
                .with_auth_method(AuthMethod::Wpa2Personal)
                .with_ssid(ssid)
                .with_password(pw);
            controller
                .set_config(&ModeConfig::Client(client_config))
                .unwrap();

            info!("Starting Wi-Fi");
            controller.start_async().await.unwrap();
            info!("Wi-Fi started!");
        }

        info!("Connecting...");
        match controller.connect_async().await {
            Ok(_) => info!("Wi-Fi connected!"),
            Err(err) => {
                error!("Connecting to Wi-Fi: {err:?}");
                delay.delay_ms(5_000).await;
            }
        }
    }

    #[cfg(feature = "access-point")]
    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                info!("Wi-Fi is running as an Access Point");

                // Until we're disconnected
                controller.wait_for_event(WifiEvent::ApStop).await;
                delay.delay_ms(5_000).await
            }
            WifiApState::Stopped | WifiApState::Invalid | _ => {
                info!(
                    "Wi-Fi Access Point status: {:?}",
                    esp_radio::wifi::ap_state()
                );
            }
        }

        if controller
            .is_started()
            .inspect_err(|err| error!("Controller is not started: {err}"))
            .unwrap_or_default()
        {
            info!("Controller is already started, waiting for connections");
            delay.delay_ms(500).await;
            continue;
        }

        let ssid = SSID.try_into().unwrap();
        let pw = PASSWORD.try_into().unwrap();
        let access_point_config = AccessPointConfig::default()
            .with_auth_method(AuthMethod::Wpa2Personal)
            .with_ssid(ssid)
            .with_password(pw);
        controller
            .set_config(&ModeConfig::AccessPoint(access_point_config))
            .unwrap();
        info!("Starting Wi-Fi Access Point");
        controller.start_async().await.unwrap();
        info!("Wi-Fi started");
    }
}
