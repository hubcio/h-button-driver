use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
use super::linux::LinuxSoundController;

#[cfg(target_os = "windows")]
use super::windows::WindowsSoundController;

#[cfg(target_os = "macos")]
use super::macos::MacOsSoundController;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum MicrophoneStatus {
    Muted,
    Unmuted,
}

pub struct SoundController {
    #[cfg(target_os = "linux")]
    sound_controller: LinuxSoundController,

    #[cfg(target_os = "windows")]
    sound_controller: WindowsSoundController,

    #[cfg(target_os = "macos")]
    sound_controller: MacOsSoundController,
}

impl SoundController {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            let sound_controller = LinuxSoundController::new();
            SoundController { sound_controller }
        }
        #[cfg(target_os = "macos")]
        {
            let sound_controller = MacOsSoundController::new();
            SoundController { sound_controller }
        }
        #[cfg(target_os = "windows")]
        {
            let sound_controller = WindowsSoundController::new();
            SoundController { sound_controller }
        }
    }

    pub fn toggle_microphone_mute(&mut self) {
        self.sound_controller.toggle_microphone_mute();
    }

    pub fn get_microphone_status(&mut self) -> MicrophoneStatus {
        self.sound_controller.get_microphone_status()
    }

    pub fn set_volume(&mut self, volume: i64) {
        println!("Setting volume to {}", volume);
        self.sound_controller.set_volume(volume);
    }

    pub fn get_current_volume(&mut self) -> i64 {
        self.sound_controller.get_current_volume()
    }

    pub fn mute_mic(&mut self) {
        println!("Muting mic");
        self.sound_controller.mute_mic();
    }

    pub fn unmute_mic(&mut self) {
        println!("Unmuting mic");
        self.sound_controller.unmute_mic();
    }
}
