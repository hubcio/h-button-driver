mod manager;
mod notifications;

use std::sync::Arc;

use btleplug::api::Characteristic;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use self::manager::BtlteManager;

#[derive(Deserialize, Serialize, Debug, PartialEq, Default)]
pub enum LedStatus {
    On,
    #[default]
    Off,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct HidStatus {
    pub encoder_position: i32,
    pub mic_mute_button_press_count: u32,
    pub led_status: LedStatus,
}

pub type OnConnectCallback = Arc<Mutex<dyn FnMut(&[u8]) -> Vec<u8> + Send + Sync>>;
pub type OnNotificationCallback = Arc<Mutex<dyn FnMut(&[u8]) -> Option<Vec<u8>> + Send + Sync>>;

pub struct ToBeNamed {
    btlte_manager: BtlteManager,
}

impl ToBeNamed {
    pub async fn new(peripheral_name_filter: &'static str) -> Self {
        let btlte_manager = BtlteManager::new(peripheral_name_filter).await;
        Self { btlte_manager }
    }

    pub async fn run(
        &mut self,
        on_connect_cb: OnConnectCallback,
        on_notification_cb: OnNotificationCallback,
    ) {
        self.btlte_manager
            .run(on_connect_cb, on_notification_cb)
            .await
            .unwrap();
    }
}

pub async fn send_message(msg: &[u8], characteristic_str: &str) {
    let msg = serde_json::to_string(&msg).unwrap().as_bytes().to_vec();
    // BtlteManager::send_message(msg, LED_STATUS_CHARACTERISTIC_UUID).await;
}

pub async fn run(
    peripheral_name_filter: &'static str,
    on_connect_cb: OnConnectCallback,
    on_notification_cb: OnNotificationCallback,
) {
    let mut manager = BtlteManager::new(peripheral_name_filter).await;
    manager
        .run(on_connect_cb, on_notification_cb)
        .await
        .unwrap();
    // block_on(manager.run());
}
