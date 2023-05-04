mod sound;

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sound::sound_controller::*;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const PERIPHERAL_NAME_MATCH_FILTER: &str = "H-Button";
const NOTIFY_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xa3c87500_8ed3_4bdf_8a39_a01bebede295);
const LED_STATUS_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x3c9a3f00_8ed3_4bdf_8a39_a01bebede295);

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum LedStatus {
    On,
    Off,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum BluetoothMessage {
    HidStatus(HidStatus), // from server (esp32) to client (windows, mac os, linux)
    SetMicMuteIndicator(LedStatus), // from client to server
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HidStatus {
    pub encoder_position: i32,
    pub mic_mute_button_press_count: u32,
    pub led_status: LedStatus,
}

// fn load_icon(path: &std::path::Path) -> tray_icon::icon::Icon {
//     let (icon_rgba, icon_width, icon_height) = {
//         let image = image::open(path)
//             .expect("Failed to open icon path")
//             .into_rgba8();
//         let (width, height) = image.dimensions();
//         let rgba = image.into_raw();
//         (rgba, width, height)
//     };
//     tray_icon::icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
//         .expect("Failed to open icon")
// }

// fn tray_init() {
//     std::thread::spawn(|| {
//         gtk::init().unwrap();
//         let event_loop = EventLoopBuilder::new().build();

//         let menu_channel = MenuEvent::receiver();
//         let tray_channel = TrayEvent::receiver();

//         let icon = load_icon(std::path::Path::new("assets/icon.png"));
//         let mut tray_menu = Menu::new();

//         let quit_i = MenuItem::new("Quit", true, None);
//         let test_i = MenuItem::new("Test", true, None);

//         tray_menu.append_items(&[
//             // &PredefinedMenuItem::about(
//             //     None,
//             //     Some(AboutMetadata {
//             //         name: Some("tao".to_string()),
//             //         copyright: Some("Copyright tao".to_string()),
//             //         ..Default::default()
//             //     }),
//             // ),
//             &test_i,
//             &PredefinedMenuItem::separator(),
//             &quit_i,
//         ]);

//         let mut tray = Some(
//             TrayIconBuilder::new()
//                 .with_menu(Box::new(tray_menu))
//                 .with_tooltip("system-tray - tray icon library!")
//                 .with_icon(icon.clone())
//                 .build()
//                 .unwrap(),
//         );

//         let second_tray_menu = Some(Menu::new());
//         // let mut second_system_tray = None;

//         println!("test");

//         event_loop.run(move |_event, _, control_flow| {
//             *control_flow = ControlFlow::Poll;

//             if let Ok(event) = menu_channel.try_recv() {
//                 if event.id == quit_i.id() {
//                     *control_flow = ControlFlow::Exit;
//                 }

//                 if event.id == test_i.id() {
//                     println!("in");

//                     // Update the menu item to "Test2".
//                     let test2_i = MenuItem::new("Test2", true, None);
//                     second_tray_menu = Some(Menu::new());
//                     tray_menu.append_items(&[&test2_i, &PredefinedMenuItem::separator(), &quit_i]);

//                     // Rebuild the tray with the new menu.
//                     tray = Some(
//                         TrayIconBuilder::new()
//                             .with_menu(Box::new(tray_menu.clone()))
//                             .with_tooltip("system-tray - tray icon library!")
//                             .with_icon(icon.clone())
//                             .build()
//                             .unwrap(),
//                     );

//                     println!("replaced");
//                 }
//                 println!("{event:?}");
//             }

//             if let Ok(event) = tray_channel.try_recv() {
//                 println!("{event:?}");
//             }
//         });
//     });
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    let mut sound_controller = SoundController::new();

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
                                let mut init_mute_press_count = 0;
                                let mut init_led_status = LedStatus::Off;
                                for _ in 0..100 {
                                    let value = peripheral.read(&characteristic).await;
                                    if let Ok(value) = value {
                                        println!(
                                            "Initial value: {:?}",
                                            String::from_utf8(value.clone())
                                        );
                                        let pb = serde_json::from_slice(&value);
                                        match pb {
                                            Ok(p) => {
                                                match p {
                                                    BluetoothMessage::HidStatus(hs) => {
                                                        init_position = hs.encoder_position;
                                                        init_mute_press_count =
                                                            hs.mic_mute_button_press_count;
                                                        init_led_status = hs.led_status;
                                                    }
                                                    _ => {
                                                        panic!("Unexpected message type")
                                                    }
                                                }
                                                true
                                            }
                                            Err(e) => {
                                                println!("Error: {:?}", e);
                                                continue;
                                            }
                                        };
                                        break;
                                    }
                                }
                                println!("Initial position: {init_position:?}, press count: {init_mute_press_count:?}, initial_led_status: {init_led_status:?}");

                                let microphone_status = sound_controller.get_microphone_status();

                                sync_led_status_with_mic_status(
                                    peripheral,
                                    &init_led_status,
                                    &microphone_status,
                                )
                                .await;

                                println!("Subscribing to characteristic {:?}", characteristic.uuid);
                                peripheral.subscribe(&characteristic).await?;

                                let mut notification_stream =
                                    peripheral.notifications().await?.take(500);

                                let mut last_press_count = init_mute_press_count;
                                let mut last_position = init_position;

                                // Process while the BLE connection is not broken or stopped.
                                while let Some(data) = notification_stream.next().await {
                                    let msg: BluetoothMessage =
                                        serde_json::from_slice(&data.value).unwrap();

                                    println!("Received bluetooth msg: {:?}", msg);

                                    match msg {
                                        BluetoothMessage::HidStatus(hs) => {
                                            if hs.mic_mute_button_press_count != last_press_count {
                                                sound_controller.toggle_microphone_mute();
                                                let msg = match sound_controller
                                                    .get_microphone_status()
                                                {
                                                    MicrophoneStatus::Muted => {
                                                        BluetoothMessage::SetMicMuteIndicator(
                                                            LedStatus::On,
                                                        )
                                                    }
                                                    MicrophoneStatus::Unmuted => {
                                                        BluetoothMessage::SetMicMuteIndicator(
                                                            LedStatus::Off,
                                                        )
                                                    }
                                                };
                                                let characteristics = peripheral.characteristics();
                                                let led_characteristic = characteristics
                                                    .iter()
                                                    .find(|c| {
                                                        c.uuid == LED_STATUS_CHARACTERISTIC_UUID
                                                            && c.properties
                                                                .contains(CharPropFlags::WRITE)
                                                    })
                                                    .unwrap();
                                                let msg = serde_json::to_string(&msg).unwrap();
                                                println!("Sending bluetooth msg: {msg}");
                                                peripheral
                                                    .write(
                                                        led_characteristic,
                                                        msg.as_bytes(),
                                                        WriteType::WithoutResponse,
                                                    )
                                                    .await
                                                    .unwrap();
                                                last_press_count = hs.mic_mute_button_press_count;
                                            }
                                            if hs.encoder_position != last_position {
                                                let volume = calculate_volume(
                                                    last_position,
                                                    hs.encoder_position,
                                                    sound_controller.get_current_volume(),
                                                );
                                                sound_controller.set_volume(volume);
                                                last_position = hs.encoder_position;
                                            }
                                        }
                                        _ => todo!(),
                                    }
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

fn calculate_volume(initial_position: i32, encoder_position: i32, current_volume: i64) -> i64 {
    let impulses_per_rotation = 240;
    let volume_range: i64 = 65536;
    let impulses = encoder_position - initial_position;

    let volume_change =
        (volume_range as f64 / impulses_per_rotation as f64 * impulses as f64) as i64;

    (current_volume + volume_change).clamp(0, volume_range)
}

async fn sync_led_status_with_mic_status<P>(
    peripheral: &P,
    init_led_status: &LedStatus,
    microphone_status: &MicrophoneStatus,
) where
    P: Peripheral,
{
    let msg = match (init_led_status, microphone_status) {
        (LedStatus::Off, MicrophoneStatus::Muted) => {
            BluetoothMessage::SetMicMuteIndicator(LedStatus::On)
        }
        (LedStatus::On, MicrophoneStatus::Unmuted) => {
            BluetoothMessage::SetMicMuteIndicator(LedStatus::Off)
        }
        _ => return,
    };

    let characteristics = peripheral.characteristics();
    let led_characteristic = characteristics
        .iter()
        .find(|c| {
            c.uuid == LED_STATUS_CHARACTERISTIC_UUID && c.properties.contains(CharPropFlags::WRITE)
        })
        .unwrap();
    let msg = serde_json::to_string(&msg).unwrap();
    println!("Sending initial sync bluetooth msg: {msg}");
    peripheral
        .write(
            led_characteristic,
            msg.as_bytes(),
            WriteType::WithoutResponse,
        )
        .await
        .unwrap();
}
