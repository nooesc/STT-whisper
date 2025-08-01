use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use device_query::{DeviceQuery, DeviceState, Keycode};
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    keybind: String,
    whisper_model_path: String,
    shortcuts: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("open terminal".to_string(), "gnome-terminal".to_string());
        shortcuts.insert("take screenshot".to_string(), "gnome-screenshot".to_string());
        shortcuts.insert("open browser".to_string(), "firefox".to_string());
        
        Settings {
            keybind: "F8".to_string(),
            whisper_model_path: "./ggml-base.en.bin".to_string(),
            shortcuts,
        }
    }
}

struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    recording: Arc<Mutex<bool>>,
}

impl AudioRecorder {
    fn new() -> Self {
        AudioRecorder {
            samples: Arc::new(Mutex::new(Vec::new())),
            recording: Arc::new(Mutex::new(false)),
        }
    }

    fn start_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        
        println!("Recording started... (sample rate: {} Hz)", sample_rate);
        
        *self.recording.lock().unwrap() = true;
        self.samples.lock().unwrap().clear();
        
        let samples_clone = Arc::clone(&self.samples);
        let recording_clone = Arc::clone(&self.recording);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        if *recording_clone.lock().unwrap() {
                            samples_clone.lock().unwrap().extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("Stream error: {}", err),
                    None
                )?
            }
            cpal::SampleFormat::I16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &_| {
                        if *recording_clone.lock().unwrap() {
                            let float_data: Vec<f32> = data.iter()
                                .map(|&s| s as f32 / i16::MAX as f32)
                                .collect();
                            samples_clone.lock().unwrap().extend_from_slice(&float_data);
                        }
                    },
                    |err| eprintln!("Stream error: {}", err),
                    None
                )?
            }
            cpal::SampleFormat::U16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &_| {
                        if *recording_clone.lock().unwrap() {
                            let float_data: Vec<f32> = data.iter()
                                .map(|&s| (s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0))
                                .collect();
                            samples_clone.lock().unwrap().extend_from_slice(&float_data);
                        }
                    },
                    |err| eprintln!("Stream error: {}", err),
                    None
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };
        
        stream.play()?;
        
        // Keep stream alive while recording
        while *self.recording.lock().unwrap() {
            thread::sleep(Duration::from_millis(100));
        }
        
        Ok(())
    }

    fn stop_recording(&self) -> Vec<f32> {
        *self.recording.lock().unwrap() = false;
        println!("Recording stopped.");
        self.samples.lock().unwrap().clone()
    }

    fn save_wav(&self, samples: &[f32], path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        
        let mut writer = WavWriter::create(path, spec)?;
        for &sample in samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        
        Ok(())
    }
}

fn transcribe_audio(whisper_path: &str, audio_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Load whisper model
    let ctx = WhisperContext::new(whisper_path)?;
    
    // Create parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_timestamps(false);
    params.set_language(Some("en"));
    
    // Load and process audio
    let mut reader = hound::WavReader::open(audio_path)?;
    let samples: Vec<f32> = reader.samples::<f32>()
        .map(|s| s.unwrap())
        .collect();
    
    // Run whisper
    ctx.full(params, &samples)?;
    
    // Get transcription
    let num_segments = ctx.full_n_segments()?;
    let mut transcription = String::new();
    
    for i in 0..num_segments {
        let segment = ctx.full_get_segment_text(i)?;
        transcription.push_str(&segment);
        transcription.push(' ');
    }
    
    Ok(transcription.trim().to_string())
}

fn execute_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Executing command: {}", command);
    
    Command::new("bash")
        .arg("-c")
        .arg(command)
        .spawn()?;
    
    Ok(())
}

fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let settings_path = "voice_assistant_settings.json";
    
    if !Path::new(settings_path).exists() {
        let default_settings = Settings::default();
        let json = serde_json::to_string_pretty(&default_settings)?;
        fs::write(settings_path, json)?;
        println!("Created default settings file: {}", settings_path);
        return Ok(default_settings);
    }
    
    let contents = fs::read_to_string(settings_path)?;
    let settings: Settings = serde_json::from_str(&contents)?;
    Ok(settings)
}

fn string_to_keycode(key: &str) -> Option<Keycode> {
    match key.to_uppercase().as_str() {
        "F1" => Some(Keycode::F1),
        "F2" => Some(Keycode::F2),
        "F3" => Some(Keycode::F3),
        "F4" => Some(Keycode::F4),
        "F5" => Some(Keycode::F5),
        "F6" => Some(Keycode::F6),
        "F7" => Some(Keycode::F7),
        "F8" => Some(Keycode::F8),
        "F9" => Some(Keycode::F9),
        "F10" => Some(Keycode::F10),
        "F11" => Some(Keycode::F11),
        "F12" => Some(Keycode::F12),
        "SPACE" => Some(Keycode::Space),
        "LCTRL" => Some(Keycode::LControl),
        "RCTRL" => Some(Keycode::RControl),
        "LSHIFT" => Some(Keycode::LShift),
        "RSHIFT" => Some(Keycode::RShift),
        "LALT" => Some(Keycode::LAlt),
        "RALT" => Some(Keycode::RAlt),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Voice Command Assistant Starting...");
    
    // Load settings
    let settings = load_settings()?;
    println!("Settings loaded successfully");
    println!("Keybind: {}", settings.keybind);
    println!("Shortcuts: {:?}", settings.shortcuts);
    
    // Check if whisper model exists
    if !Path::new(&settings.whisper_model_path).exists() {
        eprintln!("Whisper model not found at: {}", settings.whisper_model_path);
        eprintln!("Please download a ggml model from https://huggingface.co/ggerganov/whisper.cpp");
        return Err("Whisper model not found".into());
    }
    
    let device_state = DeviceState::new();
    let recorder = AudioRecorder::new();
    let mut is_recording = false;
    let target_key = string_to_keycode(&settings.keybind)
        .ok_or_else(|| format!("Invalid keybind: {}", settings.keybind))?;
    
    println!("Voice assistant ready! Press {} to start/stop recording.", settings.keybind);
    
    loop {
        let keys: Vec<Keycode> = device_state.get_keys();
        
        if keys.contains(&target_key) {
            if !is_recording {
                // Start recording
                is_recording = true;
                println!("\n🎤 Recording started...");
                
                let recorder_clone = AudioRecorder::new();
                let recorder_ref = &recorder;
                
                thread::spawn(move || {
                    if let Err(e) = recorder_ref.start_recording() {
                        eprintln!("Recording error: {}", e);
                    }
                });
                
                // Wait for key release
                while device_state.get_keys().contains(&target_key) {
                    thread::sleep(Duration::from_millis(50));
                }
            } else {
                // Stop recording and process
                is_recording = false;
                let samples = recorder.stop_recording();
                
                if !samples.is_empty() {
                    println!("Processing audio...");
                    
                    // Save audio to temporary file
                    let temp_audio = "temp_recording.wav";
                    if let Err(e) = recorder.save_wav(&samples, temp_audio) {
                        eprintln!("Failed to save audio: {}", e);
                        continue;
                    }
                    
                    // Transcribe
                    match transcribe_audio(&settings.whisper_model_path, temp_audio) {
                        Ok(text) => {
                            println!("Transcription: {}", text);
                            
                            // Check for shortcuts
                            let lower_text = text.to_lowercase();
                            let mut command_executed = false;
                            
                            for (phrase, command) in &settings.shortcuts {
                                if lower_text.contains(&phrase.to_lowercase()) {
                                    if let Err(e) = execute_command(command) {
                                        eprintln!("Failed to execute command: {}", e);
                                    } else {
                                        println!("✓ Executed: {}", phrase);
                                        command_executed = true;
                                    }
                                    break;
                                }
                            }
                            
                            if !command_executed {
                                println!("No matching shortcut found.");
                            }
                        }
                        Err(e) => eprintln!("Transcription error: {}", e),
                    }
                    
                    // Clean up temp file
                    let _ = fs::remove_file(temp_audio);
                }
                
                // Wait for key release
                while device_state.get_keys().contains(&target_key) {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        
        thread::sleep(Duration::from_millis(50));
    }
}
