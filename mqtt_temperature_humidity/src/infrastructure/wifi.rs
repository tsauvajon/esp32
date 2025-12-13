use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::rng::Rng;
use esp_radio::wifi::event::{self, EventExt};
use esp_radio::wifi::{
    AccessPointInfo, AuthMethod, ClientConfig, Interfaces, ModeConfig, ScanConfig, WifiController,
    WifiDevice, WifiEvent, WifiStaState,
};
use log::{error, info, warn};

use crate::mk_static;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

pub async fn start_wifi(
    wifi_controller: WifiController<'static>,
    interfaces: Interfaces<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    event::StaDisconnected::update_handler(|evt| {
        info!(
            "Wi-Fi STA disconnected (reason {}, RSSI {} dBm)",
            evt.reason(),
            evt.rssi()
        );
    });

    let wifi_interface = interfaces.sta;
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let net_config = embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default());

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<20>, StackResources::<20>::new()),
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
    info!("Wi-Fi link is up");

    {
        info!("Waiting to get IP address");
        loop {
            if let Some(config) = stack.config_v4() {
                info!("Network stack ready with IPv4 {}", config.address);
                break;
            }
            delay.delay_ms(100).await;
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
/// In access point mode, this starts the access point and keeps it alive (with a correct SSID and Password).
async fn connection_task(mut controller: WifiController<'static>) {
    info!(
        "Starting connection. Device capabilities: {:?}",
        controller.capabilities()
    );
    let mut delay = Delay {};

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
                info!("Wi-Fi Station status: {:?}", esp_radio::wifi::sta_state());
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

        log_scan_results(&mut controller);

        info!("Connecting...");
        match controller.connect_async().await {
            Ok(_) => info!("Wi-Fi connected!"),
            Err(err) => {
                error!("Connecting to Wi-Fi: {err:?}");
                delay.delay_ms(5_000).await;
            }
        }
    }
}

fn log_scan_results(controller: &mut WifiController<'static>) {
    match controller.scan_with_config(ScanConfig::default()) {
        Ok(access_points) => {
            if access_points.is_empty() {
                warn!("Wi-Fi scan found no access points");
                return;
            }
            info!(
                "Wi-Fi scan discovered {} access point(s)",
                access_points.len()
            );
            let mut saw_target_ssid = false;
            for ap in access_points.iter() {
                log_access_point(ap);
                if ap.ssid.as_str() == SSID {
                    saw_target_ssid = true;
                    info!(
                        "Target SSID '{}' visible on channel {} (auth {:?})",
                        SSID, ap.channel, ap.auth_method
                    );
                }
            }
            if !saw_target_ssid {
                warn!("Target SSID '{}' not found in scan results", SSID);
            }
        }
        Err(err) => warn!("Wi-Fi scan failed: {err:?}"),
    }
}

fn log_access_point(ap: &AccessPointInfo) {
    info!(
        "AP '{}' {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} ch{} RSSI {} dBm auth {:?}",
        ap.ssid,
        ap.bssid[0],
        ap.bssid[1],
        ap.bssid[2],
        ap.bssid[3],
        ap.bssid[4],
        ap.bssid[5],
        ap.channel,
        ap.signal_strength,
        ap.auth_method
    );
}
