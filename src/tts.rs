use std::sync::{Arc, Mutex};
use std::thread;
use tts::Tts;

pub struct VoiceFeedback {
    tts: Arc<Mutex<Option<Tts>>>,
    enabled: bool,
}

impl VoiceFeedback {
    pub fn new(enabled: bool) -> Self {
        let tts = if enabled {
            match Tts::default() {
                Ok(mut tts_instance) => {
                    // Set voice properties
                    let _ = tts_instance.set_rate(1.2); // Slightly faster speech
                    let _ = tts_instance.set_pitch(1.0); // Normal pitch
                    let _ = tts_instance.set_volume(0.9); // Slightly lower volume
                    Some(tts_instance)
                }
                Err(e) => {
                    eprintln!("Failed to initialize TTS: {}. Voice feedback disabled.", e);
                    None
                }
            }
        } else {
            None
        };

        VoiceFeedback {
            tts: Arc::new(Mutex::new(tts)),
            enabled,
        }
    }

    pub fn speak(&self, text: &str) {
        if !self.enabled {
            return;
        }

        let tts_clone = Arc::clone(&self.tts);
        let text = text.to_string();
        
        // Spawn a thread to avoid blocking
        thread::spawn(move || {
            if let Ok(mut tts_guard) = tts_clone.lock() {
                if let Some(ref mut tts) = *tts_guard {
                    let _ = tts.speak(&text, false);
                }
            }
        });
    }

    #[allow(dead_code)]
    pub fn speak_blocking(&self, text: &str) {
        if !self.enabled {
            return;
        }

        if let Ok(mut tts_guard) = self.tts.lock() {
            if let Some(ref mut tts) = *tts_guard {
                let _ = tts.speak(text, true);
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_speaking(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Ok(mut tts_guard) = self.tts.lock() {
            if let Some(ref mut tts) = *tts_guard {
                return tts.is_speaking().unwrap_or(false);
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        if !self.enabled {
            return;
        }

        if let Ok(mut tts_guard) = self.tts.lock() {
            if let Some(ref mut tts) = *tts_guard {
                let _ = tts.stop();
            }
        }
    }
}

impl Clone for VoiceFeedback {
    fn clone(&self) -> Self {
        VoiceFeedback {
            tts: Arc::clone(&self.tts),
            enabled: self.enabled,
        }
    }
}