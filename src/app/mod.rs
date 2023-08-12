mod poller;

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
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

#[derive(Deserialize, Serialize, Debug)]
pub enum BluetoothMessage {
    HidStatus(HidStatus), // from server (esp32) to client (windows, mac os, linux)
    SetMicMuteIndicator(LedStatus), // from client to server
}

pub struct App {
    sound_controller: Arc<Mutex<SoundController>>,
    current_hid_status: Arc<Mutex<HidStatus>>, // TODO: could be also atomics
}

impl App {
    pub fn new() -> Self {
        Self {
            sound_controller: Arc::new(Mutex::new(SoundController::new())),
            current_hid_status: Arc::new(Mutex::new(HidStatus::default())),
        }
    }
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

    // move to sound controller ?
    fn calculate_volume(initial_position: i32, encoder_position: i32, current_volume: i64) -> i64 {
        let impulses_per_rotation = 240;
        let volume_range: i64 = 65536;
        let impulses = encoder_position - initial_position;

        let volume_change =
            (volume_range as f64 / impulses_per_rotation as f64 * impulses as f64) as i64;

        (current_volume + volume_change).clamp(0, volume_range)
    }

    async fn do_something() {}

    pub async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // callback that takes raw bytes and always returns raw bytes
        // raw bytes are sent to relevant bluetooth peripheral
        let sound_controller = self.sound_controller.clone();
        let current_hs = self.current_hid_status.clone();
        let on_connect_cb = Arc::new(Mutex::new(move |msg: &[u8]| -> Vec<u8> {
            println!("Initial value: {:?}", core::str::from_utf8(msg));
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

            *current_hs.lock().unwrap() = initial_hid_status;

            let msg = match microphone_status {
                MicrophoneStatus::Muted => BluetoothMessage::SetMicMuteIndicator(LedStatus::On),
                MicrophoneStatus::Unmuted => BluetoothMessage::SetMicMuteIndicator(LedStatus::Off),
            };
            serde_json::to_string(&msg).unwrap().as_bytes().to_vec()
        }));

        // callback that takes raw bytes and maybe returns raw bytes
        // raw bytes (if returned) are sent to relevant bluetooth peripheral
        let sound_controller = self.sound_controller.clone();
        let current_hs = self.current_hid_status.clone();
        let on_notification_cb = Arc::new(Mutex::new(move |msg: &[u8]| -> Option<Vec<u8>> {
            println!("Notification: {:?}", core::str::from_utf8(msg));
            let msg = serde_json::from_slice(msg).unwrap();
            let mut sound_controller = sound_controller.lock().unwrap();
            let mut current_hs = current_hs.lock().unwrap();
            match msg {
                BluetoothMessage::HidStatus(hs) => {
                    if hs.encoder_position != current_hs.encoder_position {
                        let volume = Self::calculate_volume(
                            current_hs.encoder_position,
                            hs.encoder_position,
                            sound_controller.get_current_volume(),
                        );
                        sound_controller.set_volume(volume);
                        current_hs.encoder_position = hs.encoder_position;
                    }
                    if hs.mic_mute_button_press_count != current_hs.mic_mute_button_press_count {
                        sound_controller.toggle_microphone_mute();
                        let msg = match sound_controller.get_microphone_status() {
                            MicrophoneStatus::Muted => {
                                BluetoothMessage::SetMicMuteIndicator(LedStatus::On)
                            }
                            MicrophoneStatus::Unmuted => {
                                BluetoothMessage::SetMicMuteIndicator(LedStatus::Off)
                            }
                        };
                        current_hs.mic_mute_button_press_count = hs.mic_mute_button_press_count;
                        let msg = serde_json::to_vec(&msg).unwrap();
                        return Some(msg);
                    }
                    None
                }
                _ => panic!("Unexpected message type"),
            }
        }));

        let on_change_cb = Arc::new(
            // todo: change led status
            |microphone_status: MicrophoneStatus| match microphone_status {
                MicrophoneStatus::Muted => change_icon(TrayIcon::Muted),
                MicrophoneStatus::Unmuted => change_icon(TrayIcon::Unmuted),
            },
        );

        // let poller = poller::Poller::new(
        //     self.sound_controller.clone(),
        //     std::time::Duration::from_millis(100),
        //     on_change_cb,
        // );
        // poller.start();

        // bluetooth related code needs to be running in different OS thread
        let handle = std::thread::spawn(move || {
            block_on(ble::run(
                PERIPHERAL_NAME_MATCH_FILTER,
                on_connect_cb,
                on_notification_cb,
            ));
        });
        // Self::test_tray();
        tray_init();

        // block on the handle thread so app doesn't exit
        handle.join().unwrap();
        // init sound
        // init tray
        // init bluetooth(on_connect_cb, on_notif_cb)
        Ok(())
    }
}
