# **🎵 Pipewire Soundpad (PWSP)**

**PipeWire Soundpad (PWSP)** is a simple yet powerful **soundboard application** written in **Rust**. It provides a
user-friendly graphical interface for **managing and playing audio files, directing their output directly to the virtual
microphone.** This makes it an ideal tool for gamers, streamers, and anyone looking to inject sound effects into voice
chats on platforms like **Discord, Zoom, or Teamspeak**.

![screenshot.png](assets/screenshot.png)

# **🌟 Key Features**

* **Multi-Format Support**: Play audio files in popular formats, including _**mp3**_, _**wav**_, _**ogg**_, _**flac**_,
  _**mp4**_, and _**aac**_.
* **Virtual Microphone Output**: The application routes audio through a virtual device created by PipeWire, allowing
  other users to hear the sounds as if you were speaking into your microphone.
* **Modern and Clean GUI**: The interface is built with the [egui](https://egui.rs) library, ensuring an intuitive and
  responsive user experience.
* **Sound Collection Management**: Easily add and remove directories containing your audio files. The application scans
  these folders and displays all supported files for quick access.
* **Quick Search**: Use the built-in search bar to instantly find any sound file within your library.
* **Detailed Playback Controls**:
    * **Play/Pause button**.
    * **Volume slider** for individual sound adjustment.
    * **Position slider** to fast-forward or rewind the audio.
* **Persistent Configuration**: The list of added directories and your selected audio output device are saved
  automatically, so you won't need to reconfigure them every time you launch the application.

# **⚙️ How It Works**

PWSP is designed with a clear separation of concerns, operating through a client-server architecture. It consists of
three main components:

* **pwsp-daemon**: This is the core of the application. It runs silently in the background, managing all the
  heavy-lifting tasks. The daemon is responsible for:
    * Creating and managing virtual audio devices.
    * Linking these devices within the PipeWire graph.
    * Handling all audio playback.
* **pwsp-gui**: This is the graphical user interface. It acts as a client that communicates with pwsp-daemon via a
  **UnixSocket**. This is how you interact with your sound collection, control playback, and configure settings.
* **pwsp-cli**: This is the command-line interface, also acting as a client. It provides a way to control the daemon
  without a GUI, allowing for scripting or quick command-based actions.

# **🚀 Installation**

## **Pre-built Packages**

You can download pre-built binaries and .deb packages from
the [releases page](https://github.com/arabianq/pipewire-soundpad/releases).

## **Fedora Linux (and derivatives)**

If you're using Fedora, you can install PWSP from a dedicated repository using DNF.

Add the repository:

```bash
sudo dnf copr enable arabianq/pwsp
```

Update cache:

```bash
sudo dnf makecache
```

Install PWSP:

```bash
sudo dnf install pwsp
```

## **Arch Linux**
There is pwsp package in AUR.
You can install it using yay, paru or any other AUR helper.
```bash
paru pwsp
```

## **Installing using cargo**

```bash
cargo install pwsp
```

## **Building from source**

#### **Requirements**

* **Rust**: Install [Rust](https://www.rust-lang.org/tools/install) (using rustup is recommended).
* **PipeWire**: Ensure that [PipeWire](https://pipewire.org/) is installed and running on your system.

#### **Build Instructions**

Clone the repository:

```bash
git clone https://github.com/arabianq/pipewire-soundpad.git  
cd pipewire-soundpad
```

Build the project:

```bash
cargo build --release
```

Now you have three binary files inside ./target/release/:

- **pwsp-gui**
- **pwsp-cli**
- **pwsp-daemon**

# **🎮 Usage**

Before using pwsp-gui or pwsp-cli, you **must** first run the pwsp-daemon in the background.

### **Running the Daemon**

You can start the daemon from the terminal or enable the systemd service for automatic startup.

* **Manual Start:**

```bash
/path/to/your/pwsp-daemon &
```

* **Using systemd (recommended):**  
  If you installed PWSP using prebuilt packages, the systemd service is added automatically.
    1. **Start the service:**
        ```bash  
        systemctl --user start pwsp-daemon
        ```
    2. **Enable autostart (starts on login):**
        ```bash
       systemctl --user enable --now pwsp-daemon
        ```

### **Using the GUI**

1. **Add Sounds**: Click the **"+"** button and select a folder containing your audio files. The application
   will automatically list all supported files.
2. **Select Microphone**: In the main application window, select your microphone. PWSP will automatically
   create a virtual microphone and feed it sound from two sources: **your microphone** and the **audio files**.
3. **Playback**: Click on a file in the list to load it, then use the **"Play"** and **"Pause"** buttons to control
   playback. You can also play single file once using **"Play File"** button.

### **Using the CLI**

The pwsp-cli tool allows you to control the daemon from the command line.

* **General Help**: To see a list of all available commands, run:

```bash
pwsp-cli --help
```

* **Example Commands**:
    * **Play a file**:

      ```bash
      pwsp-cli action play <file_path>
      ```

    * **Get the current volume**:

      ```bash
      pwsp-cli get volume
      ```

    * **Set playback position to 20 seconds**:

      ```bash 
      pwsp-cli set position 20
      ```

### **Hotkeys & Controls**

#### **Keyboard Shortcuts**

| Key                      | Action                                               |
| :----------------------- | :--------------------------------------------------- |
| **Esc**                  | Close application                                    |
| **Space**                | Pause / Resume audio                                 |
| **Backspace**            | Stop all audio tracks                                |
| **Enter**                | Play selected file (stops all other tracks)          |
| **Ctrl + Enter**         | Add selected file to playback (plays simultaneously) |
| **Shift + Enter**        | Replace the last added track with the selected one   |
| **I**                    | Open / Close settings                                |
| **/**                    | Focus search field                                   |
| **Ctrl + ↑ / ↓**         | Navigate through files                               |
| **Ctrl + Shift + ↑ / ↓** | Navigate through directories                         |

#### **Mouse Controls**

* **Left Click**: Play track (stops all other tracks).
* **Ctrl + Left Click**: Add track (plays simultaneously with current tracks).
* **Shift + Left Click**: Replace the last added track with the selected one.

# **🤝 Contributing**

Contributions are welcome\! If you have ideas for improvements or find a bug, feel free to create
an [issue](https://github.com/arabianq/pipewire-soundpad/issues) or submit
a [pull request](https://github.com/arabianq/pipewire-soundpad/pulls).

# **📜 License**

This project is licensed under
the [MIT License](https://github.com/arabianq/pipewire-soundpad/blob/main/LICENSE).
