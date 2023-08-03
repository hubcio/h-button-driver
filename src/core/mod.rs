use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use tauri::async_runtime::block_on;
use uuid::Uuid;

use crate::{
    ble::{self, *},
    sound::sound_controller::*,
    tray::{tray_menu::tray_init, *},
};

const PERIPHERAL_NAME_MATCH_FILTER: &str = "H-Button";
const NOTIFY_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xa3c87500_8ed3_4bdf_8a39_a01bebede295);
const LED_STATUS_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x3c9a3f00_8ed3_4bdf_8a39_a01bebede295);

fn test_tray() {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        change_icon(TrayIcon::Muted);
        change_title_test1();
        std::thread::sleep(std::time::Duration::from_secs(1));
        change_icon(TrayIcon::Unmuted);
        change_title_test2();
        std::thread::sleep(std::time::Duration::from_secs(1));
    });
}

fn calculate_volume(initial_position: i32, encoder_position: i32, current_volume: i64) -> i64 {
    let impulses_per_rotation = 240;
    let volume_range: i64 = 65536;
    let impulses = encoder_position - initial_position;

    let volume_change =
        (volume_range as f64 / impulses_per_rotation as f64 * impulses as f64) as i64;

    (current_volume + volume_change).clamp(0, volume_range)
}

async fn do_something() {}

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    // dummy function to init async runtime
    do_something().await;
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let sound_controller = Arc::new(Mutex::new(SoundController::new()));

    let on_connect_cb = Arc::new(Mutex::new(move |msg: &[u8]| -> Vec<u8> {
        println!("Initial value: {:?}", String::from_utf8(msg.to_vec()));
        let pb = serde_json::from_slice(msg);
        let initial_hid_status = match pb {
            Ok(p) => match p {
                BluetoothMessage::HidStatus(hs) => Ok(hs),
                _ => {
                    panic!("Unexpected message type")
                }
            },
            Err(e) => {
                println!("Error: {:?}", e);
                Err(e)
            }
        }
        .unwrap();

        let microphone_status = sound_controller.lock().unwrap().get_microphone_status();
        println!(
            "Initial hid status: {:?}, microphone: {:?}",
            initial_hid_status, microphone_status
        );
        let msg = match microphone_status {
            MicrophoneStatus::Muted => BluetoothMessage::SetMicMuteIndicator(LedStatus::On),
            MicrophoneStatus::Unmuted => BluetoothMessage::SetMicMuteIndicator(LedStatus::Off),
        };
        serde_json::to_string(&msg).unwrap().as_bytes().to_vec()
    }));

    // let on_notification_cb = Arc::new(Mutex::new(move |msg: &[u8]| {
    //     println!("Notification: {:?}", String::from_utf8(msg.to_vec()));

    //     match msg {
    //         BluetoothMessage::HidStatus(hs) => {
    //             if hs.mic_mute_button_press_count != last_press_count {
    //                 sound_controller.toggle_microphone_mute();
    //                 let msg = match sound_controller.get_microphone_status() {
    //                     MicrophoneStatus::Muted => {
    //                         BluetoothMessage::SetMicMuteIndicator(LedStatus::On)
    //                     }
    //                     MicrophoneStatus::Unmuted => {
    //                         BluetoothMessage::SetMicMuteIndicator(LedStatus::Off)
    //                     }
    //                 };
    //                 let characteristics = peripheral.characteristics();
    //                 let led_characteristic = characteristics
    //                     .iter()
    //                     .find(|c| {
    //                         c.uuid == LED_STATUS_CHARACTERISTIC_UUID
    //                             && c.properties.contains(CharPropFlags::WRITE)
    //                     })
    //                     .unwrap();
    //                 let msg = serde_json::to_string(&msg).unwrap();
    //                 println!("Sending bluetooth msg: {msg}");
    //                 peripheral
    //                     .write(
    //                         led_characteristic,
    //                         msg.as_bytes(),
    //                         WriteType::WithoutResponse,
    //                     )
    //                     .await
    //                     .unwrap();
    //                 last_press_count = hs.mic_mute_button_press_count;
    //             }
    //             if hs.encoder_position != last_position {
    //                 let volume = calculate_volume(
    //                     last_position,
    //                     hs.encoder_position,
    //                     sound_controller.get_current_volume(),
    //                 );
    //                 sound_controller.set_volume(volume);
    //                 last_position = hs.encoder_position;
    //             }
    //         }
    //         _ => todo!(),
    //     }
    // }));

    // bluetooth related code needs to be running in different OS thread
    let handle = std::thread::spawn(move || {
        block_on(ble::run(PERIPHERAL_NAME_MATCH_FILTER, on_connect_cb));
    });
    test_tray();
    tray_init();

    // block on the handle thread so app doesn't exit
    handle.join().unwrap();
    // init sound
    // init tray
    // init bluetooth(on_connect_cb, on_notif_cb)
    Ok(())
}
