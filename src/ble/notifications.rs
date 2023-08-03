use crate::ble::BluetoothMessage;

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

use super::Callback;

pub enum NotificationsManagerCommand {
    Stop,
}

pub(crate) struct NotificationsManager<T: PeripheralTrait + 'static> {
    peripheral: T,
    tx: Arc<Mutex<Sender<NotificationsManagerCommand>>>,
    rx: Arc<Mutex<Receiver<NotificationsManagerCommand>>>,
    on_connect_callback: Callback,
}

impl<T: PeripheralTrait> NotificationsManager<T> {
    pub async fn new(peripheral: T, callback: Callback) -> Self {
        let (tx, rx): (
            Sender<NotificationsManagerCommand>,
            Receiver<NotificationsManagerCommand>,
        ) = channel(256);

        Self {
            peripheral,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            on_connect_callback: callback,
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
                let on_connect_callback = self.on_connect_callback.clone();
                let initial_data = peripheral.read(&characteristic).await.unwrap();
                let msg = on_connect_callback.lock().unwrap()(&initial_data);
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
                    select!(
                        _ = async {
                            loop {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                trace!("Worker is alive");
                            }
                        }.fuse() => {},
                        _ = async {
                            while let Some(data) = notification_stream.next().await {
                                let msg: BluetoothMessage = serde_json::from_slice(&data.value).unwrap();
                                println!("Received bluetooth msg: {:?}", msg);


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



    fn mainloop(&self) {
        // let mut last_press_count = init_mute_press_count;
        // let mut last_position = init_position;
        // // Process while the BLE connection is not broken or stopped.
        // let microphone_status = sound_controller.get_microphone_status();

        // while let Some(data) = notification_stream.next().await {
        //     let msg: BluetoothMessage = serde_json::from_slice(&data.value).unwrap();

        //     println!("Received bluetooth msg: {:?}", msg);

        // }
    }
}
