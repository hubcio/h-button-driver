use super::sound_controller::MicrophoneStatus;

pub struct WindowsSoundController {
    current_microphone_status: MicrophoneStatus,
}

impl WindowsSoundController {
    pub fn new() -> Self {
        todo!();
    }

    pub fn get_microphone_status(&self) -> MicrophoneStatus {
        todo!();
    }

    pub fn toggle_microphone_mute(&mut self) {
        todo!();
    }

    pub fn set_volume(&mut self, volume: i64) {
        todo!();
    }

    pub fn get_current_volume(&mut self) -> i64 {
        todo!();
    }

    pub fn mute_mic(&mut self) {
        todo!();
    }

    pub fn unmute_mic(&mut self) {
        todo!();
    }
}
