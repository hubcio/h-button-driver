mod manager;
mod notifications;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use self::manager::BtlteManager;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum LedStatus {
    On,
    Off,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HidStatus {
    pub encoder_position: i32,
    pub mic_mute_button_press_count: u32,
    pub led_status: LedStatus,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum BluetoothMessage {
    HidStatus(HidStatus), // from server (esp32) to client (windows, mac os, linux)
    SetMicMuteIndicator(LedStatus), // from client to server
}

pub type Callback = Arc<Mutex<dyn FnMut(&[u8]) -> Vec<u8> + Send + Sync>>;
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

pub async fn run(peripheral_name_filter: &'static str, on_connect_cb: Callback) {
    let mut manager = BtlteManager::new(peripheral_name_filter).await;
    manager.run(on_connect_cb).await.unwrap();
    // block_on(manager.run());
}
