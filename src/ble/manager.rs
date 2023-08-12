use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use std::error::Error;
use tokio::sync::oneshot::Receiver;
use uuid::Uuid;

use super::notifications::NotificationsManager;
use super::{OnConnectCallback, OnNotificationCallback};

pub struct BtlteManager {
    _manager: Manager,
    adapter: Adapter,
    notifications_manager: Option<NotificationsManager<Peripheral>>,
    connected_peripheral: Option<PeripheralId>,
    peripheral_name_filter: &'static str,
}

impl BtlteManager {
    pub async fn new(peripheral_name_filter: &'static str) -> Self {
        let manager = Manager::new().await.unwrap();
        let adapter = Self::get_central(&manager).await;
        let notifications_manager = None;
        Self {
            _manager: manager,
            adapter,
            notifications_manager,
            connected_peripheral: None,
            peripheral_name_filter,
        }
    }

    pub(crate) async fn run(
        &mut self,
        on_connect_cb: OnConnectCallback,
        on_notification_cb: OnNotificationCallback,
    ) -> Result<(), Box<dyn Error>> {
        let mut events = self.adapter.events().await?;
        self.adapter.start_scan(ScanFilter::default()).await?;

        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    if let Some(_valid_peripheral) = self.is_valid_peripheral(&id).await {
                        println!("Valid DeviceDiscovered: {:?}", id);
                        self.connect(&id).await;
                    }
                }
                CentralEvent::DeviceConnected(id) => {
                    if let Some(valid_peripheral) = self.is_valid_peripheral(&id).await {
                        println!("DeviceConnected: {:?}", id);

                        let notifications_manager = NotificationsManager::new(
                            valid_peripheral,
                            on_connect_cb.clone(),
                            on_notification_cb.clone(),
                        )
                        .await;

                        // send msg to notifications manager to start
                        notifications_manager.start().await;
                        self.notifications_manager = Some(notifications_manager);
                    }
                }
                CentralEvent::DeviceDisconnected(id) => {
                    if let Some(_valid_peripheral) = self.is_valid_peripheral(&id).await {
                        println!("Device disconnected, stopping notifications manager, attempting reconnect {:?}", id);

                        // stop notifications loop
                        let notifications_manager = self.notifications_manager.as_mut();
                        notifications_manager.unwrap().stop().await;
                        self.notifications_manager = None;
                        self.connect(&id).await;
                    }
                }
                event => {
                    trace!("Unhandled btleplug central event: {:?}", event)
                }
            }
        }
        Ok(())
    }

    async fn get_central(manager: &Manager) -> Adapter {
        let adapters = manager.adapters().await.unwrap();
        adapters.into_iter().next().unwrap()
    }

    async fn is_valid_peripheral(&self, peripheral: &PeripheralId) -> Option<Peripheral> {
        let peripheral = self.adapter.peripheral(peripheral).await.unwrap();

        let local_name = peripheral
            .properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .unwrap_or(String::from("(peripheral name unknown)"));

        if local_name.contains(self.peripheral_name_filter) {
            return Some(peripheral);
        }
        None
    }

    async fn connect(&mut self, peripheral: &PeripheralId) {
        let peripheral = self.adapter.peripheral(peripheral).await.unwrap();
        let is_connected = peripheral.is_connected().await.unwrap();
        if is_connected {
            return;
        }
        self.connected_peripheral = Some(peripheral.id());
        peripheral.connect().await.unwrap();
    }

    async fn _disconnect(&self, peripheral: &PeripheralId) {
        let peripheral = self.adapter.peripheral(peripheral).await.unwrap();
        let is_connected = peripheral.is_connected().await.unwrap();
        if !is_connected {
            return;
        }
        peripheral.disconnect().await.unwrap();
    }

    // move to notifications manager
    pub async fn send_message(&self, msg: &[u8], characteristic: Uuid) {
        let connected_peripheral = self.connected_peripheral.clone().unwrap();
        let peripheral = self
            .adapter
            .peripheral(&connected_peripheral)
            .await
            .unwrap();
        let characteristics = peripheral.characteristics();
        let led_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == characteristic && c.properties.contains(CharPropFlags::WRITE))
            .unwrap();
        println!("Sending bluetooth msg: {msg:?}");
        peripheral
            .write(led_characteristic, msg, WriteType::WithoutResponse)
            .await
            .unwrap();
    }
}
