# STT-Whisper Voice Assistant

A Rust-based voice command assistant that uses OpenAI's Whisper model for speech-to-text transcription and allows you to trigger custom commands with your voice.

## Features

- **Hotkey Activation**: Press and hold F8 (configurable) to record voice commands
- **Local Speech Recognition**: Uses Whisper model for accurate offline transcription
- **Custom Voice Shortcuts**: Define phrases that trigger specific commands
- **Cross-platform Audio**: Built with cpal for audio recording
- **JSON Configuration**: Easy-to-edit settings file

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Whisper Model**: Download a GGML format model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp)
   - Recommended: `ggml-base.en.bin` for English-only recognition
   - Place the model file in the project root directory

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd STT-whisper
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

The application creates a `voice_assistant_settings.json` file on first run with default settings:

```json
{
  "keybind": "F8",
  "whisper_model_path": "./ggml-base.en.bin",
  "shortcuts": {
    "open terminal": "gnome-terminal",
    "take screenshot": "gnome-screenshot",
    "open browser": "firefox"
  }
}
```

### Available Keybinds
- F1-F12 function keys
- SPACE, LCTRL, RCTRL, LSHIFT, RSHIFT, LALT, RALT

### Adding Custom Shortcuts
Edit the `shortcuts` section to add your own voice commands:
```json
"shortcuts": {
  "open editor": "code",
  "system monitor": "htop",
  "lock screen": "gnome-screensaver-command -l"
}
```

## Usage

1. Run the application:
```bash
cargo run --release
```

2. Press and hold the configured hotkey (default: F8)
3. Speak your command
4. Release the hotkey to process the recording
5. The assistant will transcribe your speech and execute matching commands

## Dependencies

- `cpal`: Cross-platform audio I/O
- `device_query`: Keyboard input detection
- `hound`: WAV file handling
- `whisper-rs`: Rust bindings for Whisper
- `serde` & `serde_json`: Configuration serialization

## Troubleshooting

- **No input device**: Ensure your microphone is connected and permissions are granted
- **Model not found**: Download the whisper model and update the path in settings
- **CUDA support**: The project is configured for CUDA acceleration. Remove the "cuda" feature from Cargo.toml if you don't have a compatible GPU

## License

[Add your license information here]