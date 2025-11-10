use core::{convert::identity, net::Ipv4Addr, str::FromStr};

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::rng::Rng;
use esp_radio::wifi::{
    AccessPointConfig, AuthMethod, Interfaces, ModeConfig, WifiApState, WifiController, WifiDevice,
    WifiEvent,
};
use log::info;

use crate::mk_static;

// When in station mode: remote WiFi credentials
// When in access point mode: credentials for clients to use
const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

// Access point
const STATIC_IP: &str = "192.168.13.37/24";
const GATEWAY_IP: &str = "192.168.13.37";

pub async fn start_wifi(
    wifi_controller: WifiController<'static>,
    interfaces: Interfaces<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    let wifi_interface = interfaces.ap;
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let address = Ipv4Cidr::from_str(STATIC_IP).unwrap();
    let gateway = Some(Ipv4Addr::from_str(GATEWAY_IP).unwrap());
    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address,
        gateway,
        dns_servers: Default::default(),
    });

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(connection_task(wifi_controller)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();

    wait_for_connection(stack).await;

    stack
}

async fn wait_for_connection(stack: Stack<'_>) {
    let mut delay = Delay {};
    info!("Waiting for link to be up");
    while !stack.is_link_up() {
        delay.delay_ms(500).await;
    }

    while !stack.is_config_up() {
        delay.delay_ms(100).await;
    }

    stack
        .config_v4()
        .inspect(|config| info!("ipv4 config: {config:?}"));

    // ### Station mode ###
    // info!("Waiting to get IP address");
    // loop {
    //     if let Some(config) = stack.config_v4() {
    //         info!("Got IP: {}", config.address);
    //         break;
    //     }
    //     delay.delay_ms(100).await;
    // }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!(
        "Starting connection task. Device capabilities: {:?}",
        controller.capabilities()
    );

    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                // Until we're disconnected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Delay {}.delay_ms(5_000).await
            }
            WifiApState::Stopped | WifiApState::Invalid | _ => {}
        }

        if !controller.is_started().is_ok_and(identity) {
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
        info!("Starting WiFi Access Point");
        controller.start_async().await.unwrap();
        info!("WiFi started");
    }
}
