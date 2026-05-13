<div align="center">
  <h1>🎵 PipeWire Soundpad (PWSP)</h1>
  <p><b>A simple, modern, and powerful soundboard for Linux, written in Rust.</b></p>
  <img src="assets/screenshot.png" alt="PWSP Screenshot" width="700"/>
</div>

## 🌟 Overview
**PipeWire Soundpad (PWSP)** is a graphical soundboard application that routes audio directly to your virtual microphone using **PipeWire**. It provides an intuitive interface for managing your audio collection, making it an ideal tool for gamers, streamers, and anyone looking to inject sound effects into voice chats on platforms like Discord, Zoom, or TeamSpeak.

## ✨ Key Features
* **🎙️ Virtual Microphone Output:** Seamlessly mixes your microphone input with sound effects by automatically managing PipeWire virtual devices.
* **🎵 Multi-Format Support:** Plays popular audio formats including `mp3`, `wav`, `ogg`, `flac`, `mp4`, and `aac`.
* **⚡ Global Hotkeys:** Trigger sounds instantly from anywhere, even when the app is running in the background.
* **📂 Smart Collection Management:** Drag-and-drop folders, quick search, and collapsible tracks to keep your library organized.
* **🎛️ Advanced Playback Controls:** Individual volume sliders, play/pause, position scrubbing, and concurrent multi-track playback.
* **🔌 Plug & Play:** Automatically detects when an input device is connected or disconnected and handles linking/unlinking on the fly.
* **🖥️ Modern GUI:** Clean, responsive, and lightweight interface powered by [egui](https://egui.rs/).

## ⚙️ Architecture
PWSP is built with a client-server model to ensure stability and separation of concerns:
* **`pwsp-daemon`**: The background engine. It runs silently, managing PipeWire virtual devices, audio routing, and playback.
* **`pwsp-gui`**: The graphical interface. Communicates with the daemon via a Unix socket to control playback and settings.
* **`pwsp-cli`**: The command-line tool. Perfect for scripting, hotkey binding, or quick terminal-based control.

---

## 🚀 Installation

### 📦 Flatpak (Recommended)
Install PWSP via Flatpak from our custom repository:
```bash
flatpak remote-add --user --if-not-exists pwsp-repo https://arabianq.github.io/pipewire-soundpad/index.flatpakrepo

# Install stable version
flatpak install --user arabianq-repo ru.arabianq.pwsp//stable

# Or install the nightly version (latest commit)
flatpak install --user arabianq-repo ru.arabianq.pwsp//nightly
```

### 🐧 Linux Packages
**Fedora (and derivatives):**
```bash
sudo dnf copr enable arabianq/pwsp
sudo dnf install pwsp
```

**Arch Linux (AUR):**
```bash
paru -S pwsp-bin # or 'pwsp' to build from source
```

**Debian / Ubuntu:** 
Download pre-built `.deb` packages or standalone binaries from the [Releases page](https://github.com/arabianq/pipewire-soundpad/releases).

### 🦀 Cargo / Source Build
```bash
cargo install pwsp

# OR clone and build manually:
git clone https://github.com/arabianq/pipewire-soundpad.git
cd pipewire-soundpad
cargo build --release
```
*(Note: Requires Rust toolchain and PipeWire running on your system).*

---

## 🎮 Usage

### 1. Start the Daemon
Before using the GUI or CLI, the daemon must be running in the background.

```bash
# Recommended: Start and enable via systemd (starts on login)
systemctl --user enable --now pwsp-daemon

# Manual start (if not using systemd):
pwsp-daemon &
```

### 2. Using the GUI
1. **Add Sounds:** Click the **"+"** button to add a directory containing your audio files.
2. **Select Mic:** Choose your physical microphone from the dropdown. PWSP will instantly create a virtual microphone combining your voice and the soundboard.
3. **Play:** Click any sound to play it, adjust its volume, or assign a hotkey for quick access.

### 3. Using the CLI
Control the daemon directly from your terminal:
```bash
pwsp-cli action play /path/to/sound.mp3
pwsp-cli get volume
pwsp-cli set position 20
pwsp-cli --help # View all commands
```

---

## ⌨️ Shortcuts & Controls

| Action                               | Keyboard               | Mouse                |
| :----------------------------------- | :--------------------- | :------------------- |
| **Play Track** (Stops others)        | `Enter`                | `Left Click`         |
| **Add Track** (Plays simultaneously) | `Ctrl + Enter`         | `Ctrl + Left Click`  |
| **Replace Last Track**               | `Shift + Enter`        | `Shift + Left Click` |
| **Pause / Resume**                   | `Space`                |                      |
| **Stop All Tracks**                  | `Backspace`            |                      |
| **Open / Close Settings**            | `I`                    |                      |
| **Search**                           | `/`                    |                      |
| **Navigate Files**                   | `Ctrl + ↑ / ↓`         |                      |
| **Navigate Directories**             | `Ctrl + Shift + ↑ / ↓` |                      |

---

## 🤝 Contributing
Contributions, issues, and feature requests are welcome! Feel free to check out the [issues page](https://github.com/arabianq/pipewire-soundpad/issues).

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/arabianq/pipewire-soundpad)

## 📜 License
This project is licensed under the [MIT License](LICENSE).