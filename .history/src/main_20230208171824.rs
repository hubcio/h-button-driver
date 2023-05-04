// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use alsa::Mixer;

/// Only devices whose name contains this string will be tried.
const PERIPHERAL_NAME_MATCH_FILTER: &str = "H-Button";
/// UUID of the characteristic for which we should subscribe to notifications.
const NOTIFY_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xa3c87500_8ed3_4bdf_8a39_a01bebede295);

#[derive(Deserialize)]
struct BluetoothPayload {
    position: i32,
    mute: bool,
}

// async fn alsa_test() -> Result<(), Box<dyn Error>> {
//     let selem = mixer
//         .find_selem(&alsa::mixer::SelemId::new("Master", 0))
//         .unwrap();

//     // for i in 0..65536 {
//     //     if i % 1000 == 0 {
//     //         println!("Setting volume to {i}");
//     //         selem.set_playback_volume_all(i as i64).unwrap();
//     //         // sleep 1 ms
//     //         time::sleep(Duration::from_millis(100)).await;
//     //     }
//     // }

//     let selem = mixer
//         .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
//         .unwrap()
//         .set_capture_switch_all(0)
//         .unwrap();

//     selem.set_capture_switch_all(1).unwrap();

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // "Battery Service" and "HID Service"
    pretty_env_logger::init();

    // alsa_test().await.unwrap();

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        println!("Starting scan...");
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(2)).await;
        let peripherals = adapter.peripherals().await?;

        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range.
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                println!(
                    "Peripheral {:?} is connected: {:?}",
                    &local_name, is_connected
                );
                // Check if it's the peripheral we want.
                if local_name.contains(PERIPHERAL_NAME_MATCH_FILTER) {
                    println!("Found matching peripheral {:?}...", &local_name);
                    if !is_connected {
                        // Connect if we aren't already connected.
                        if let Err(err) = peripheral.connect().await {
                            eprintln!("Error connecting to peripheral, skipping: {err}");
                            continue;
                        }
                    }
                    let is_connected = peripheral.is_connected().await?;
                    println!(
                        "Now connected ({:?}) to peripheral {:?}.",
                        is_connected, &local_name
                    );
                    if is_connected {
                        println!("Discover peripheral {local_name:?} services...");
                        peripheral.discover_services().await?;

                        // read position and set is a base

                        for characteristic in peripheral.characteristics() {
                            println!("Checking characteristic {characteristic:?}");
                            // Subscribe to notifications from the characteristic with the selected
                            // UUID.
                            if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
                                && characteristic.properties.contains(CharPropFlags::NOTIFY)
                            {
                                // try 100 times to read the initial value and return it from closure, if not return default values
                                let mut init_position = 0;
                                let mut init_mute = false;
                                for _ in 0..100 {
                                    let value = peripheral.read(&characteristic).await;
                                    if let Ok(value) = value {
                                        let pb = serde_json::from_slice(&value);
                                        match pb {
                                            Ok(p) => {
                                                let BluetoothPayload { position, mute } = p;
                                                init_position = position;
                                                init_mute = mute;
                                                true
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        };
                                        break;
                                    }
                                }
                                println!("Initial position: {init_position:?}, initial mute: {init_mute:?}");

                                println!("Subscribing to characteristic {:?}", characteristic.uuid);
                                peripheral.subscribe(&characteristic).await?;

                                let mut notification_stream =
                                    peripheral.notifications().await?.take(500);

                                // Process while the BLE connection is not broken or stopped.
                                while let Some(data) = notification_stream.next().await {
                                    println!(
                                        "Received data: {}",
                                        String::from_utf8_lossy(&data.value)
                                    );

                                    let BluetoothPayload { position, mute } =
                                        serde_json::from_slice(&data.value).unwrap();

                                    let mixer = Mixer::new("default", false).unwrap();

                                    // let current_volume = mixer
                                    //     .find_selem(&alsa::mixer::SelemId::new("Master", 0))
                                    //     .unwrap()
                                    //     .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
                                    //     .unwrap();

                                    mixer
                                        .find_selem(&alsa::mixer::SelemId::new("Master", 0))
                                        .unwrap()
                                        .set_playback_volume_all(
                                            ((position * 1024) - (init_position * 1024)) as i64,
                                        )
                                        .unwrap();

                                    mixer
                                        .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
                                        .unwrap()
                                        .set_capture_switch_all(mute as i32)
                                        .unwrap();

                                    // print current volume, position and mute
                                    println!(
                                        "Current position: {position:?}, current mute: {mute:?}"
                                    );
                                }
                            }
                        }
                        println!("Disconnecting from peripheral {local_name:?}...");
                        peripheral.disconnect().await?;
                    }
                } else {
                    println!("Skipping unknown peripheral {peripheral:?}");
                }
            }
        }
    }
    Ok(())
}
