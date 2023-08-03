use alsa::{
    mixer::{SelemChannelId, SelemId},
    Mixer,
};

use super::sound_controller::MicrophoneStatus;

pub struct LinuxSoundController {
    mixer: Mixer,
    current_microphone_status: MicrophoneStatus,
}

impl LinuxSoundController {
    pub fn new() -> Self {
        let mixer = Mixer::new("default", false).unwrap();
        let sound = mixer
            .find_selem(&SelemId::new("Capture", 0))
            .unwrap()
            .get_capture_switch(SelemChannelId::mono())
            .unwrap();

        let current_microphone_status = if sound == 0 {
            MicrophoneStatus::Muted
        } else {
            MicrophoneStatus::Unmuted
        };

        LinuxSoundController {
            mixer,
            current_microphone_status,
        }
    }

    pub fn get_microphone_status(&self) -> MicrophoneStatus {
        let sound = self
            .mixer
            .find_selem(&SelemId::new("Capture", 0))
            .unwrap()
            .get_capture_switch(SelemChannelId::mono())
            .unwrap();
        if sound != 0 {
            println!("Microphone is unmuted, sound = {sound}");
            MicrophoneStatus::Unmuted
        } else {
            println!("Microphone is muted, sound = {sound}");
            MicrophoneStatus::Muted
        }
    }

    pub fn toggle_microphone_mute(&mut self) {
        match self.get_microphone_status() {
            MicrophoneStatus::Muted => {
                self.unmute_mic();
                self.current_microphone_status = MicrophoneStatus::Unmuted;
            }
            MicrophoneStatus::Unmuted => {
                self.mute_mic();
                self.current_microphone_status = MicrophoneStatus::Muted;
            }
        }
    }

    pub fn set_volume(&mut self, volume: i64) {
        println!("Setting volume to {}", volume);

        self.mixer
            .find_selem(&SelemId::new("Master", 0))
            .unwrap()
            .set_playback_volume_all(volume)
            .unwrap();
    }

    pub fn get_current_volume(&mut self) -> i64 {
        let current_volume = self
            .mixer
            .find_selem(&SelemId::new("Master", 0))
            .unwrap()
            .get_playback_volume(SelemChannelId::mono())
            .unwrap();

        println!("current volume {current_volume}");
        current_volume
    }

    pub fn mute_mic(&mut self) {
        println!("Muting mic");
        self.mixer
            .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
            .unwrap()
            .set_capture_volume(SelemChannelId::mono(), 0)
            .unwrap();
    }

    pub fn unmute_mic(&mut self) {
        println!("Unmuting mic");
        self.mixer
            .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
            .unwrap()
            .set_capture_volume(SelemChannelId::mono(), 65536)
            .unwrap();
    }
}
