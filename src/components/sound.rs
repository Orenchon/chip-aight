use kira::{
    arrangement::{handle::ArrangementHandle, Arrangement, LoopArrangementSettings},
    instance::{InstanceSettings, PauseInstanceSettings, ResumeInstanceSettings},
    sound::{Sound, SoundSettings},
};
use kira::{
    manager::{AudioManager, AudioManagerSettings},
    sound::handle::SoundHandle,
};
pub struct SoundManager {
    audio_manager: AudioManager,
    sound_handle: SoundHandle,
    arrangement_handle: ArrangementHandle,
}
impl SoundManager {
    pub fn new() -> Result<SoundManager, &'static str> {
        let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
        let sound_handle_result =
            audio_manager.load_sound("data/beep.wav", SoundSettings::default());
        match sound_handle_result {
            Ok(sound_handle) => {
                //sound_handle.play(InstanceSettings::default()).unwrap();
                let mut arrangement_handle = audio_manager
                    .add_arrangement(Arrangement::new_loop(
                        &sound_handle,
                        LoopArrangementSettings::default(),
                    ))
                    .unwrap();
                arrangement_handle.play(InstanceSettings::default());
                arrangement_handle.pause(PauseInstanceSettings::default());
                return Ok(SoundManager {
                    audio_manager,
                    sound_handle,
                    arrangement_handle,
                });
            }
            Err(err) => return Err("Failed to load data/beep.wav"),
        }
    }
    pub fn play(&mut self) {
        self.arrangement_handle
            .resume(ResumeInstanceSettings::default());
        return ();
    }
    pub fn pause(&mut self) {
        self.arrangement_handle
            .pause(PauseInstanceSettings::default());
        return ();
    }
}
