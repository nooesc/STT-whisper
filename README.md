# STT-Whisper Voice Assistant

A cross-platform voice command assistant that uses OpenAI's Whisper model for accurate speech-to-text transcription. Control your computer with voice commands - launch applications, take screenshots, open files, and execute custom commands, all triggered by a simple hotkey.

## What is this for?

This tool is designed for:
- **Hands-free computer control**: Execute commands without touching your keyboard
- **Accessibility**: Help users who have difficulty with traditional input methods
- **Productivity**: Quickly launch frequently used applications or run complex commands
- **Automation**: Create voice-triggered shortcuts for repetitive tasks
- **Custom workflows**: Build your own voice-controlled automation system

## Features

- **Cross-platform**: Works on macOS, Linux, and Windows
- **Hotkey Activation**: Press and hold F8 (configurable) to record voice commands
- **Local Speech Recognition**: Uses Whisper model for accurate offline transcription
- **Custom Voice Shortcuts**: Define your own phrases to trigger any command
- **JSON Configuration**: Easy-to-edit settings file
- **No Internet Required**: Everything runs locally on your machine

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Whisper Model**: Download a GGML format model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp)
   - Recommended: `ggml-base.en.bin` for English-only recognition
   - Place the model file in the project root directory

### Platform-specific Requirements

#### macOS
- Grant microphone permissions when prompted
- Works out of the box with system audio

#### Linux
- Install ALSA development libraries:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libasound2-dev
  
  # Fedora
  sudo dnf install alsa-lib-devel
  
  # Arch
  sudo pacman -S alsa-lib
  ```

#### Windows
- No additional requirements
- Windows Defender may prompt on first run

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd STT-whisper
```

2. Download the Whisper model:
```bash
# Download the base English model (recommended)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

3. Build the project:
```bash
cargo build --release
```

## Usage

1. Run the application:
```bash
cargo run --release
```

2. The application will create a `voice_assistant_settings.json` file on first run

3. Press and hold **F8** to start recording
4. Speak your command clearly
5. Release **F8** to process the command
6. The assistant will transcribe your speech and execute matching commands

## Creating Custom Voice Commands

The real power of STT-Whisper comes from creating your own custom voice commands. Here's how:

### 1. Edit the Settings File

Open `voice_assistant_settings.json` in your favorite text editor:

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

### 2. Add Your Own Commands

Add new entries to the `shortcuts` object:

```json
{
  "keybind": "F8",
  "whisper_model_path": "./ggml-base.en.bin",
  "shortcuts": {
    "open terminal": "gnome-terminal",
    "take screenshot": "gnome-screenshot",
    "open browser": "firefox",
    "open email": "thunderbird",
    "lock screen": "gnome-screensaver-command -l",
    "show calendar": "gnome-calendar",
    "play music": "spotify",
    "open code editor": "code",
    "system monitor": "gnome-system-monitor"
  }
}
```

### 3. Command Examples by Platform

#### macOS Commands
```json
{
  "shortcuts": {
    "open safari": "open -a Safari",
    "open music": "open -a Music",
    "empty trash": "osascript -e 'tell application \"Finder\" to empty the trash'",
    "show desktop": "osascript -e 'tell application \"System Events\" to key code 103 using {command down, shift down}'",
    "open downloads": "open ~/Downloads",
    "sleep computer": "pmset sleepnow",
    "increase volume": "osascript -e 'set volume output volume (output volume of (get volume settings) + 10)'",
    "decrease volume": "osascript -e 'set volume output volume (output volume of (get volume settings) - 10)'"
  }
}
```

#### Linux Commands
```json
{
  "shortcuts": {
    "open files": "nautilus || dolphin || thunar",
    "show processes": "gnome-system-monitor || ksysguard",
    "open calculator": "gnome-calculator || kcalc || xcalc",
    "lock screen": "gnome-screensaver-command -l || xdg-screensaver lock",
    "shutdown": "systemctl poweroff",
    "restart": "systemctl reboot",
    "open settings": "gnome-control-center || systemsettings",
    "take full screenshot": "gnome-screenshot || spectacle || scrot ~/screenshot.png"
  }
}
```

#### Windows Commands
```json
{
  "shortcuts": {
    "open notepad": "notepad",
    "open calculator": "calc",
    "lock computer": "rundll32.exe user32.dll,LockWorkStation",
    "open task manager": "taskmgr",
    "open control panel": "control",
    "empty recycle bin": "powershell.exe -command \"Clear-RecycleBin -Force\"",
    "open downloads": "explorer %USERPROFILE%\\Downloads",
    "system info": "msinfo32"
  }
}
```

### 4. Advanced Command Examples

#### Multi-step Commands
```json
{
  "shortcuts": {
    "backup documents": "tar -czf ~/backup-$(date +%Y%m%d).tar.gz ~/Documents",
    "git status": "cd ~/projects && git status",
    "update system": "sudo apt update && sudo apt upgrade -y",
    "clean downloads": "find ~/Downloads -type f -mtime +30 -delete"
  }
}
```

#### Application Launchers
```json
{
  "shortcuts": {
    "open slack": "slack || /Applications/Slack.app/Contents/MacOS/Slack",
    "open zoom": "zoom || /Applications/zoom.us.app/Contents/MacOS/zoom.us",
    "open discord": "discord || /Applications/Discord.app/Contents/MacOS/Discord",
    "start recording": "obs || /Applications/OBS.app/Contents/MacOS/OBS"
  }
}
```

#### Web Shortcuts
```json
{
  "shortcuts": {
    "open youtube": "xdg-open https://youtube.com || open https://youtube.com",
    "open github": "xdg-open https://github.com || open https://github.com",
    "check weather": "xdg-open https://weather.com || open https://weather.com",
    "open documentation": "xdg-open https://docs.rust-lang.org || open https://docs.rust-lang.org"
  }
}
```

### 5. Tips for Creating Commands

1. **Keep phrases short and distinct**: "open terminal" is better than "please open the terminal application"

2. **Use natural language**: Choose phrases you'll remember easily

3. **Test commands first**: Run the command in your terminal to ensure it works before adding it

4. **Use full paths when needed**: Some applications may need full paths to execute properly

5. **Chain commands**: Use `&&` to run multiple commands in sequence, `||` for fallbacks

6. **Consider context**: You can create commands that change directory first: `cd ~/projects && code .`

### 6. Changing the Activation Key

To change the hotkey from F8 to another key, edit the `keybind` field:

```json
{
  "keybind": "F12",  // Changed from F8 to F12
  ...
}
```

Available keys:
- Function keys: `F1` through `F12`
- Modifier keys: `LCTRL`, `RCTRL`, `LSHIFT`, `RSHIFT`, `LALT`, `RALT`
- `SPACE`

## Troubleshooting

### No input device available
- Ensure your microphone is connected and permissions are granted
- On macOS: Check System Preferences > Security & Privacy > Microphone
- On Linux: Check if your user is in the `audio` group: `sudo usermod -a -G audio $USER`

### Whisper model not found
- Download the model and ensure the path in settings matches the file location
- Use absolute paths if relative paths don't work

### Command not executing
- Test the command directly in your terminal first
- Check for typos in the settings file
- Ensure the application has necessary permissions

### Poor recognition accuracy
- Speak clearly and at a moderate pace
- Reduce background noise
- Try a larger Whisper model for better accuracy
- Ensure your microphone is working properly

## Performance Tips

- Use the `ggml-base.en.bin` model for the best balance of speed and accuracy
- Smaller models (`tiny`, `small`) are faster but less accurate
- Larger models (`medium`, `large`) are more accurate but slower
- The first transcription may be slower as the model loads into memory

## Security Considerations

- This tool can execute any command you configure
- Be cautious about what commands you add to your shortcuts
- Avoid commands that require passwords or sensitive information
- Review your settings file regularly

## License

[Add your license information here]

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.