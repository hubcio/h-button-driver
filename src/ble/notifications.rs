use crate::app::BluetoothMessage;

use btleplug::api::{CharPropFlags, Peripheral as PeripheralTrait, WriteType};
use futures::stream::StreamExt;
use futures::FutureExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc::channel, mpsc::Receiver, mpsc::Sender, Mutex};

const NOTIFY_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xa3c87500_8ed3_4bdf_8a39_a01bebede295);
const LED_STATUS_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x3c9a3f00_8ed3_4bdf_8a39_a01bebede295);

use uuid::Uuid;

use super::{OnConnectCallback, OnNotificationCallback};

pub enum NotificationsManagerCommand {
    Stop,
}

pub(crate) struct NotificationsManager<T: PeripheralTrait + 'static> {
    peripheral: T,
    tx: Arc<Mutex<Sender<NotificationsManagerCommand>>>,
    rx: Arc<Mutex<Receiver<NotificationsManagerCommand>>>,
    on_connect_cb: OnConnectCallback,
    on_notification_cb: OnNotificationCallback,
}

impl<T: PeripheralTrait> NotificationsManager<T> {
    pub async fn new(
        peripheral: T,
        on_connect_cb: OnConnectCallback,
        on_notification_cb: OnNotificationCallback,
    ) -> Self {
        let (tx, rx): (
            Sender<NotificationsManagerCommand>,
            Receiver<NotificationsManagerCommand>,
        ) = channel(256);

        Self {
            peripheral,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            on_connect_cb,
            on_notification_cb,
        }
    }

    pub async fn start(&self) {
        self.peripheral.discover_services().await.unwrap();

        // Read position and set is a base
        for characteristic in self.peripheral.characteristics() {
            info!("Checking characteristic {characteristic:?}");
            // Subscribe to notifications from the characteristic with the selected UUID
            if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
                && characteristic.properties.contains(CharPropFlags::NOTIFY)
            {
                println!("Subscribing to characteristic {:?}", characteristic.uuid);
                self.peripheral.subscribe(&characteristic).await.unwrap();
                let peripheral = self.peripheral.clone();
                let rx: Arc<Mutex<Receiver<NotificationsManagerCommand>>> = self.rx.clone();
                let on_connect_cb = self.on_connect_cb.clone();
                let on_notification_cb = self.on_notification_cb.clone();
                let initial_data = peripheral.read(&characteristic).await.unwrap();
                let msg = on_connect_cb.lock().unwrap()(&initial_data);
                let characteristics = peripheral.characteristics();
                let led_characteristic = characteristics
                    .iter()
                    .find(|c| {
                        c.uuid == LED_STATUS_CHARACTERISTIC_UUID
                            && c.properties.contains(CharPropFlags::WRITE)
                    })
                    .unwrap();
                peripheral
                    .write(led_characteristic, &msg, WriteType::WithoutResponse)
                    .await
                    .unwrap();

                tokio::spawn(async move {
                    let mut notification_stream = peripheral.notifications().await.unwrap();
                    let on_notification_cb = on_notification_cb.clone();
                    select!(
                        _ = async {
                            loop {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                trace!("Worker is alive");
                            }
                        }.fuse() => {},
                        _ = async {
                            while let Some(data) = notification_stream.next().await {
                                let in_msg: BluetoothMessage = serde_json::from_slice(&data.value).unwrap();
                                println!("Received bluetooth msg: {:?}", in_msg);
                                let out_msg = on_notification_cb.lock().unwrap()(&data.value);
                                if let Some(out_msg) = out_msg
                                {
                                    let characteristics = peripheral.characteristics();
                                    let led_characteristic = characteristics
                                        .iter()
                                        .find(|c| {
                                            c.uuid == LED_STATUS_CHARACTERISTIC_UUID
                                                && c.properties.contains(CharPropFlags::WRITE)
                                        })
                                        .unwrap();
                                    println!("Sending bluetooth msg: {msg:?}");
                                    peripheral
                                        .write(
                                            led_characteristic,
                                            &out_msg,
                                            WriteType::WithoutResponse,
                                        ).await.unwrap();
                                }
                            }
                        }.fuse() => {},
                        _ = async {
                            while let Some(cmd) = rx.lock().await.recv().await {
                                match cmd {
                                    NotificationsManagerCommand::Stop => {
                                        println!("Stopping notifications manager");
                                    }
                                }
                            }
                        }.fuse() => {},
                    );
                });
            }
        }
    }

    pub async fn stop(&self) {
        let tx = self.tx.lock().await;
        tx.send(NotificationsManagerCommand::Stop).await.unwrap();
    }
}
