use alsa::{
    mixer::{SelemChannelId, SelemId},
    Mixer,
};

use super::sound_controller::MicrophoneStatus;

pub struct LinuxSoundController {
    current_microphone_status: MicrophoneStatus,
}

impl LinuxSoundController {
    pub fn new() -> Self {
        let mixer = Mixer::new("default", false).unwrap();
        let mut channels = SelemChannelId::all().iter();
        let sound = mixer
            .find_selem(&SelemId::new("Capture", 0))
            .unwrap()
            .get_capture_switch(*channels.next().unwrap())
            .unwrap();

        let current_microphone_status = if sound == 0 {
            MicrophoneStatus::Muted
        } else {
            MicrophoneStatus::Unmuted
        };

        LinuxSoundController {
            current_microphone_status,
        }
    }

    pub fn get_microphone_status(&self) -> MicrophoneStatus {
        let mixer = Mixer::new("default", false).unwrap();
        let mut channels = SelemChannelId::all().iter();

        let selem = mixer.find_selem(&SelemId::new("Capture", 0)).unwrap();
        let capture_switch_state = selem.get_capture_switch(*channels.next().unwrap()).unwrap();
        let volume = selem.get_capture_volume(*channels.next().unwrap()).unwrap();

        println!("switch: {}, volume: {}", capture_switch_state, volume);

        let state = match (volume, capture_switch_state) {
            (0, _) => MicrophoneStatus::Muted,
            (_, 0) => MicrophoneStatus::Muted,
            (_, _) => MicrophoneStatus::Unmuted,
        };

        println!("Microphone status: {:?}", state);
        state
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
        let mixer = Mixer::new("default", false).unwrap();

        println!("Setting volume to {}", volume);
        mixer
            .find_selem(&SelemId::new("Master", 0))
            .unwrap()
            .set_playback_volume_all(volume)
            .unwrap();
    }

    pub fn get_current_volume(&mut self) -> i64 {
        let mixer = Mixer::new("default", false).unwrap();

        let current_volume = mixer
            .find_selem(&SelemId::new("Master", 0))
            .unwrap()
            .get_playback_volume(SelemChannelId::mono())
            .unwrap();

        println!("Current volume: {current_volume}");
        current_volume
    }

    pub fn mute_mic(&mut self) {
        let mixer = Mixer::new("default", false).unwrap();

        println!("Muting mic");
        mixer
            .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
            .unwrap()
            .set_capture_switch_all(0)
            .unwrap();
    }

    pub fn unmute_mic(&mut self) {
        let mixer = Mixer::new("default", false).unwrap();
        println!("Unmuting mic");
        mixer
            .find_selem(&alsa::mixer::SelemId::new("Capture", 0))
            .unwrap()
            .set_capture_switch_all(65536)
            .unwrap();
    }
}
