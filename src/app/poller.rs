// periodically poll the sound card for new data
// if the microphone is muted, then light the led on device
// if the microphone is unmuted, then turn off the led device
// use callbacks

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::sound::sound_controller::{MicrophoneStatus, SoundController};

pub struct Poller {
    sound_controller: Arc<Mutex<SoundController>>,
    poll_timeout: Duration,
    on_change_cb: Arc<dyn Fn(MicrophoneStatus) + Send + Sync>,
}

impl Poller {
    pub fn new(
        sound_controller: Arc<Mutex<SoundController>>,
        poll_timeout: Duration,
        on_change_cb: Arc<dyn Fn(MicrophoneStatus) + Send + Sync>,
    ) -> Self {
        Poller {
            sound_controller,
            poll_timeout,
            on_change_cb,
        }
    }

    pub fn start(&self) {
        println!("Poller is running");
        let sound_controller = self.sound_controller.clone();
        let poll_timeout = self.poll_timeout;
        let on_change_cb = self.on_change_cb.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            loop {
                let microphone_status = sound_controller.lock().unwrap().get_microphone_status();
                println!("Microphone status: {:?}", microphone_status);
                on_change_cb(microphone_status);
                tokio::time::sleep(poll_timeout).await;
            }
        });
    }

    pub fn stop(&self) {
        println!("Poller is stopping");
    }
}
