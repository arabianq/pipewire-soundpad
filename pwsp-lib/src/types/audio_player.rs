use crate::{
    types::pipewire::{AudioDevice, DeviceType},
    utils::{
        daemon::with_daemon_config,
        pipewire::{
            PwTerminator, create_link, ensure_route, get_all_devices, get_device, get_device_by_id,
            get_sink,
        },
    },
};
use anyhow::{Result, anyhow};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

const VIRTUAL_MIC_NAME: &str = "pwsp-virtual-mic";

/// How long we wait for a freshly opened rodio stream to show up in the PipeWire graph.
///
/// Streams are re-opened on every play that follows an idle period, so this sits directly
/// in the path between pressing a key and hearing the sound. The node normally appears
/// within one or two polls; the generous attempt count only bounds the pathological case.
const NODE_DISCOVERY_ATTEMPTS: u32 = 200;
const NODE_DISCOVERY_INTERVAL: Duration = Duration::from_millis(5);

type FileDecoder = Decoder<BufReader<fs::File>>;

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum PlayerState {
    #[default]
    Stopped,
    Paused,
    Playing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: u32,
    pub path: PathBuf,
    pub duration: Option<f32>,
    pub position: f32,
    pub volume: f32,
    pub looped: bool,
    pub paused: bool,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FullState {
    pub state: PlayerState,
    pub tracks: Vec<TrackInfo>,
    pub monitoring_volume: f32,
    pub mic_volume: f32,
    pub volume_multiplier: f32,
    pub current_input: String,
    pub all_inputs: HashMap<String, String>,
    pub current_output: String,
    pub all_outputs: HashMap<String, String>,
}

/// Which of the two independent output paths a volume applies to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VolumeTarget {
    /// What the user hears locally.
    Monitoring,
    /// What is fed into the virtual microphone, i.e. what everybody else hears.
    Mic,
}

/// The two rodio players backing a single track — one per output path.
///
/// Every playback control has to reach both, so they are wrapped here instead of being
/// fanned out by hand at each call site.
pub struct PlayerPair {
    pub monitoring: Player,
    pub mic: Option<Player>,
}

impl PlayerPair {
    pub fn for_each(&self, f: impl Fn(&Player)) {
        f(&self.monitoring);
        if let Some(mic) = &self.mic {
            f(mic);
        }
    }

    /// The player that answers queries about position and paused state for the pair.
    pub fn primary(&self) -> &Player {
        &self.monitoring
    }

    /// True once every player of the pair has run out of samples.
    pub fn empty(&self) -> bool {
        self.monitoring.empty() && self.mic.as_ref().is_none_or(|mic| mic.empty())
    }
}

pub struct PlayingSound {
    pub id: u32,
    pub players: PlayerPair,
    pub path: PathBuf,
    pub duration: Option<f32>,
    pub looped: bool,
    pub volume: f32,
}

/// Final linear gain handed to a rodio `Player`.
///
/// Values above `1.0` are intentionally allowed — amplification is the whole point of the
/// mic path. Anything non-finite or negative collapses to silence rather than blowing up
/// the mixer.
pub fn effective_gain(master: f32, track: f32, multiplier: f32) -> f32 {
    let gain = master * track * multiplier;
    if gain.is_finite() && gain > 0.0 {
        gain
    } else {
        0.0
    }
}

pub struct AudioPlayer {
    monitoring_stream: Option<MixerDeviceSink>,
    mic_stream: Option<MixerDeviceSink>,

    /// PipeWire nodes backing the two streams, once discovered.
    monitoring_node: Option<AudioDevice>,
    mic_node: Option<AudioDevice>,

    /// Links we created ourselves; dropping them tears the routes down.
    monitoring_route: Option<PwTerminator>,
    mic_route: Option<PwTerminator>,
    input_link_sender: Option<PwTerminator>,

    pub tracks: HashMap<u32, PlayingSound>,
    pub next_id: u32,

    pub input_device_name: Option<String>,
    /// `None` means "follow the system default sink" — we then leave routing of the
    /// monitoring stream to WirePlumber instead of pinning it ourselves.
    pub output_device_name: Option<String>,

    pub monitoring_volume: f32,
    pub mic_volume: f32,
    pub volume_multiplier: f32,
}

impl AudioPlayer {
    pub async fn new() -> Result<Self> {
        let (
            default_input_name,
            default_output_name,
            default_monitoring_volume,
            default_mic_volume,
            default_volume_multiplier,
        ) = with_daemon_config(|c| {
            (
                c.default_input_name.clone(),
                c.default_output_name.clone(),
                c.default_monitoring_volume.unwrap_or(1.0),
                c.default_mic_volume.unwrap_or(1.0),
                c.default_volume_multiplier.unwrap_or(1.0),
            )
        });

        let mut audio_player = AudioPlayer {
            monitoring_stream: None,
            mic_stream: None,
            monitoring_node: None,
            mic_node: None,
            monitoring_route: None,
            mic_route: None,
            input_link_sender: None,

            tracks: HashMap::new(),
            next_id: 1,

            input_device_name: default_input_name,
            output_device_name: default_output_name,

            monitoring_volume: default_monitoring_volume,
            mic_volume: default_mic_volume,
            volume_multiplier: default_volume_multiplier,
        };

        if audio_player.input_device_name.is_some() {
            audio_player.link_devices().await?;
        }

        Ok(audio_player)
    }

    // ---------- Streams ----------

    /// Opens both output streams and routes them, if that has not happened yet.
    ///
    /// Streams are opened one at a time: node discovery below tells them apart by diffing
    /// the graph around each open, which only works if the opens do not overlap.
    async fn ensure_streams(&mut self) -> Result<()> {
        if self.monitoring_stream.is_none() {
            let (stream, node) = open_stream_and_identify().await?;
            self.monitoring_stream = Some(stream);
            self.monitoring_node = node;
        }

        if self.mic_stream.is_none() {
            let (stream, node) = open_stream_and_identify().await?;
            self.mic_stream = Some(stream);
            self.mic_node = node;
        }

        self.ensure_routes().await;

        Ok(())
    }

    /// Closes both output streams once nothing is playing.
    ///
    /// This is what lets a laptop suspend: an open stream keeps the audio device busy and
    /// blocks sleep. The routing is rebuilt from scratch on the next `play()`, since
    /// re-opening mints new PipeWire node ids.
    fn drop_streams(&mut self) {
        if self.monitoring_stream.is_none() && self.mic_stream.is_none() {
            return;
        }

        self.monitoring_stream = None;
        self.mic_stream = None;
        // The nodes and our links die with the streams, so none of this may outlive them.
        self.monitoring_node = None;
        self.mic_node = None;
        self.monitoring_route = None;
        self.mic_route = None;
    }

    /// Re-asserts both routes. Idempotent, so it can run on every device-check tick to
    /// undo any re-linking WirePlumber did behind our back.
    async fn ensure_routes(&mut self) {
        if let Some(node) = self.mic_node.clone() {
            match self.route(&node, VirtualMicTarget).await {
                Ok(Some(terminator)) => self.mic_route = Some(terminator),
                Ok(None) => {}
                Err(err) => eprintln!("Failed to route mic stream to virtual mic: {}", err),
            }
        }

        // With no explicit output device the monitoring stream is left to WirePlumber,
        // which keeps PWSP following whatever the system default sink is.
        if let Some(name) = self.output_device_name.clone()
            && let Some(node) = self.monitoring_node.clone()
        {
            match self.route(&node, SinkTarget(&name)).await {
                Ok(Some(terminator)) => self.monitoring_route = Some(terminator),
                Ok(None) => {}
                Err(err) => eprintln!(
                    "Failed to route monitoring stream to output device {}: {}",
                    name, err
                ),
            }
        }
    }

    async fn route<T: RouteTarget>(
        &self,
        source: &AudioDevice,
        target: T,
    ) -> Result<Option<PwTerminator>> {
        let source = get_device_by_id(source.id).await?;
        let target = target.resolve().await?;
        ensure_route(&source, &target).await
    }

    // ---------- PipeWire links ----------

    fn abort_link_thread(&mut self) {
        if self.input_link_sender.is_some() {
            println!("Sent terminate signal to input link thread");
            self.input_link_sender = None;
        }
    }

    async fn link_devices(&mut self) -> Result<()> {
        self.abort_link_thread();

        let input_device;
        if let Some(input_device_name) = &self.input_device_name {
            if let Ok(device) = get_device(input_device_name).await {
                input_device = device;
            } else {
                eprintln!(
                    "Could not find selected input device {}, skipping device linking",
                    input_device_name
                );
                return Ok(());
            }
        } else {
            eprintln!("No input device selected, skipping device linking");
            return Ok(());
        }

        let daemon_input;
        if let Ok(device) = get_device(VIRTUAL_MIC_NAME).await {
            daemon_input = device;
        } else {
            eprintln!("Could not find pwsp-virtual-mic device, skipping device linking");
            return Ok(());
        }

        let Some(output_fl) = input_device.output_fl.clone() else {
            eprintln!("Failed to get input device output_fl");
            return Ok(());
        };
        let Some(output_fr) = input_device.output_fr.clone() else {
            eprintln!("Failed to get input device output_fr");
            return Ok(());
        };
        let Some(input_fl) = daemon_input.input_fl.clone() else {
            eprintln!("Failed to get pwsp-virtual-mic input_fl");
            return Ok(());
        };
        let Some(input_fr) = daemon_input.input_fr.clone() else {
            eprintln!("Failed to get pwsp-virtual-mic input_fr");
            return Ok(());
        };

        self.input_link_sender = Some(create_link(output_fl, output_fr, input_fl, input_fr).await?);

        Ok(())
    }

    // ---------- Transport ----------

    pub fn pause(&mut self, id: Option<u32>) {
        self.for_selected(id, |sound| sound.players.for_each(|p| p.pause()));
    }

    pub fn resume(&mut self, id: Option<u32>) {
        self.for_selected(id, |sound| sound.players.for_each(|p| p.play()));
    }

    fn for_selected(&mut self, id: Option<u32>, f: impl Fn(&mut PlayingSound)) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                f(sound);
            }
        } else {
            for sound in self.tracks.values_mut() {
                f(sound);
            }
        }
    }

    pub fn stop(&mut self, id: Option<u32>) {
        if let Some(id) = id {
            self.tracks.remove(&id);
        } else {
            self.tracks.clear();
        }
        if self.tracks.is_empty() {
            self.drop_streams();
        }
    }

    pub fn is_paused(&self) -> bool {
        if self.tracks.is_empty() {
            return false;
        }
        self.tracks
            .values()
            .all(|s| s.players.primary().is_paused())
    }

    pub fn get_state(&self) -> PlayerState {
        if self.tracks.is_empty() {
            return PlayerState::Stopped;
        }

        if self
            .tracks
            .values()
            .any(|s| !s.players.primary().is_paused() && !s.players.primary().empty())
        {
            return PlayerState::Playing;
        }

        if self.is_paused() {
            return PlayerState::Paused;
        }

        PlayerState::Stopped
    }

    // ---------- Volume ----------

    pub fn get_volume(&self, id: Option<u32>) -> Option<f32> {
        match id {
            Some(id) => self.tracks.get(&id).map(|sound| sound.volume),
            None => Some(self.monitoring_volume),
        }
    }

    /// Pushes the current master/track/multiplier state onto every live player.
    ///
    /// Single source of truth for the gain math — every setter below funnels through it.
    fn reapply_volumes(&mut self) {
        for sound in self.tracks.values() {
            sound.players.monitoring.set_volume(effective_gain(
                self.monitoring_volume,
                sound.volume,
                self.volume_multiplier,
            ));
            if let Some(mic) = &sound.players.mic {
                mic.set_volume(effective_gain(
                    self.mic_volume,
                    sound.volume,
                    self.volume_multiplier,
                ));
            }
        }
    }

    pub fn set_master_volume(&mut self, volume: f32, target: VolumeTarget) {
        match target {
            VolumeTarget::Monitoring => self.monitoring_volume = volume,
            VolumeTarget::Mic => self.mic_volume = volume,
        }
        self.reapply_volumes();
    }

    pub fn get_master_volume(&self, target: VolumeTarget) -> f32 {
        match target {
            VolumeTarget::Monitoring => self.monitoring_volume,
            VolumeTarget::Mic => self.mic_volume,
        }
    }

    pub fn set_volume_multiplier(&mut self, multiplier: f32) {
        self.volume_multiplier = multiplier;
        self.reapply_volumes();
    }

    /// Sets a single track's volume, or — without an id — moves both masters at once,
    /// which is the "make everything quieter" shortcut.
    pub fn set_volume(&mut self, volume: f32, id: Option<u32>) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.volume = volume;
            }
        } else {
            self.monitoring_volume = volume;
            self.mic_volume = volume;
        }
        self.reapply_volumes();
    }

    // ---------- Position ----------

    pub fn get_position(&self, id: Option<u32>) -> f32 {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get(&id) {
                return sound.players.primary().get_pos().as_secs_f32();
            }
        } else if let Some(sound) = self.tracks.values().last() {
            // Fallback to last added track if no ID
            return sound.players.primary().get_pos().as_secs_f32();
        }
        0.0
    }

    pub fn seek(&mut self, position: f32, id: Option<u32>) -> Result<()> {
        let position = if position < 0.0 { 0.0 } else { position };
        let position = Duration::from_secs_f32(position);

        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.players.monitoring.try_seek(position)?;
                if let Some(mic) = &sound.players.mic {
                    mic.try_seek(position).ok();
                }
            }
        } else {
            for sound in self.tracks.values_mut() {
                sound.players.for_each(|p| {
                    p.try_seek(position).ok();
                });
            }
        }
        Ok(())
    }

    pub fn get_duration(&mut self, id: Option<u32>) -> Result<f32> {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get(&id) {
                return sound.duration.ok_or(anyhow!("Unknown duration"));
            }
        } else if let Some(sound) = self.tracks.values().last() {
            return sound.duration.ok_or(anyhow!("Unknown duration"));
        }
        Err(anyhow!("No track playing"))
    }

    // ---------- Playback ----------

    pub async fn play(&mut self, file_path: &Path, concurrent: bool) -> Result<u32> {
        // One decoder per output path: the two streams run on different device clocks,
        // so they cannot share a source anyway, and rodio's `Buffered` (the only shareable
        // source) does not support seeking.
        let (monitoring_source, mic_source) = tokio::try_join!(
            decode(file_path.to_path_buf()),
            decode(file_path.to_path_buf()),
        )?;

        if !concurrent {
            self.tracks.clear();
        }

        self.ensure_streams().await?;

        let duration = monitoring_source.total_duration().map(|d| d.as_secs_f32());

        let monitoring_mixer = self
            .monitoring_stream
            .as_ref()
            .ok_or_else(|| anyhow!("monitoring stream is unexpectedly missing"))?
            .mixer();
        let monitoring = Player::connect_new(monitoring_mixer);
        monitoring.set_volume(effective_gain(
            self.monitoring_volume,
            1.0,
            self.volume_multiplier,
        ));
        monitoring.append(monitoring_source);
        monitoring.play();

        // A missing mic stream degrades to monitoring-only playback rather than failing.
        let mic = self.mic_stream.as_ref().map(|stream| {
            let player = Player::connect_new(stream.mixer());
            player.set_volume(effective_gain(self.mic_volume, 1.0, self.volume_multiplier));
            player.append(mic_source);
            player.play();
            player
        });

        let id = self.next_id;
        self.next_id += 1;

        self.tracks.insert(
            id,
            PlayingSound {
                id,
                players: PlayerPair { monitoring, mic },
                path: file_path.to_path_buf(),
                duration,
                looped: false,
                volume: 1.0,
            },
        );

        Ok(id)
    }

    pub fn set_loop(&mut self, enabled: bool, id: Option<u32>) {
        self.for_selected(id, |sound| sound.looped = enabled);
    }

    pub fn get_tracks(&self) -> Vec<TrackInfo> {
        let mut tracks: Vec<_> = self
            .tracks
            .values()
            .map(|sound| TrackInfo {
                id: sound.id,
                path: sound.path.clone(),
                duration: sound.duration,
                position: sound.players.primary().get_pos().as_secs_f32(),
                volume: sound.volume,
                looped: sound.looped,
                paused: sound.players.primary().is_paused(),
            })
            .collect();
        tracks.sort_by_key(|t| t.id);
        tracks
    }

    pub async fn update(&mut self, check_devices: bool) {
        if check_devices {
            if let Some(input_device_name) = &self.input_device_name {
                // Unlink devices if selected input device was removed
                if self.input_link_sender.is_some() && get_device(input_device_name).await.is_err()
                {
                    eprintln!(
                        "Selected input device {} was removed, unlinking devices",
                        input_device_name
                    );
                    self.abort_link_thread();
                }
                // Link devices if not linked
                else if self.input_link_sender.is_none() {
                    self.link_devices().await.ok();
                }
            }

            if self.monitoring_stream.is_some() || self.mic_stream.is_some() {
                self.ensure_routes().await;
            }
        }

        self.restart_looped_tracks().await;

        self.tracks
            .retain(|_, sound| !sound.players.empty() || sound.looped);

        if self.tracks.is_empty() {
            self.drop_streams();
        }
    }

    async fn restart_looped_tracks(&mut self) {
        let restarts: Vec<u32> = self
            .tracks
            .iter()
            .filter(|(_, sound)| sound.looped && sound.players.empty())
            .map(|(id, _)| *id)
            .collect();

        let mut restart_futures = vec![];
        for id in restarts {
            if let Some(sound) = self.tracks.get(&id) {
                let path = sound.path.clone();
                restart_futures.push(async move {
                    tokio::try_join!(decode(path.clone()), decode(path))
                        .ok()
                        .map(|(monitoring, mic)| (id, monitoring, mic))
                });
            }
        }

        for future in restart_futures {
            if let Some((id, monitoring_source, mic_source)) = future.await
                && let Some(sound) = self.tracks.get_mut(&id)
            {
                if sound.players.monitoring.empty() {
                    sound.players.monitoring.append(monitoring_source);
                    sound.players.monitoring.play();
                }
                if let Some(mic) = &sound.players.mic
                    && mic.empty()
                {
                    mic.append(mic_source);
                    mic.play();
                }
            }
        }
    }

    // ---------- Device selection ----------

    pub async fn set_current_input_device(&mut self, name: &str) -> Result<()> {
        let input_device = get_device(name).await?;

        if input_device.device_type != DeviceType::Input {
            return Err(anyhow!("Selected device is not an input device"));
        }

        self.input_device_name = Some(name.to_string());

        self.link_devices().await?;

        Ok(())
    }

    pub async fn set_current_output_device(&mut self, name: &str) -> Result<()> {
        // Fails early with a useful message if the name does not name a sink.
        get_sink(name).await?;

        self.output_device_name = Some(name.to_string());
        self.monitoring_route = None;
        self.ensure_routes().await;

        Ok(())
    }
}

/// Where a stream should be routed. Resolved late so a device that came back after a
/// reconnect is picked up on the next tick.
trait RouteTarget {
    async fn resolve(&self) -> Result<AudioDevice>;
}

struct VirtualMicTarget;

impl RouteTarget for VirtualMicTarget {
    async fn resolve(&self) -> Result<AudioDevice> {
        get_device(VIRTUAL_MIC_NAME).await
    }
}

struct SinkTarget<'a>(&'a str);

impl RouteTarget for SinkTarget<'_> {
    async fn resolve(&self) -> Result<AudioDevice> {
        get_sink(self.0).await
    }
}

async fn decode(path: PathBuf) -> Result<FileDecoder> {
    tokio::task::spawn_blocking(move || {
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path.display()));
        }
        let file = fs::File::open(&path)?;
        Decoder::try_from(file).map_err(|e| anyhow!(e))
    })
    .await?
}

/// True for the `Stream/Output/Audio` nodes that belong to us.
///
/// The guard matters: node discovery below hands the result to link pruning, and pruning
/// a stranger's node would silence another application.
fn is_own_stream_node(device: &AudioDevice) -> bool {
    let is_pwsp = |s: &str| s.to_ascii_lowercase().contains("pwsp");
    (is_pwsp(&device.name) || is_pwsp(&device.nick))
        && device.output_fl.is_some()
        && device.output_fr.is_some()
}

async fn own_stream_node_ids() -> HashSet<u32> {
    match get_all_devices().await {
        Ok(devices) => devices
            .outputs
            .iter()
            .filter(|d| is_own_stream_node(d))
            .map(|d| d.id)
            .collect(),
        Err(_) => HashSet::new(),
    }
}

/// Opens one rodio output stream and works out which PipeWire node it produced.
///
/// Identification is a before/after diff of our own stream nodes rather than an index
/// into a sorted list, so it does not depend on enumeration order. Callers must open
/// streams one at a time for the diff to stay unambiguous.
///
/// A stream that cannot be matched to a node yields `None`: playback still works, only
/// explicit routing is skipped.
async fn open_stream_and_identify() -> Result<(MixerDeviceSink, Option<AudioDevice>)> {
    let before = own_stream_node_ids().await;

    let mut stream = DeviceSinkBuilder::open_default_sink()?;
    stream.log_on_drop(false);

    // Checked before the first sleep: the PCM is already open by the time open_stream
    // returns, so the node is often registered and this costs nothing.
    for attempt in 0..NODE_DISCOVERY_ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(NODE_DISCOVERY_INTERVAL).await;
        }

        let devices = match get_all_devices().await {
            Ok(devices) => devices,
            Err(_) => continue,
        };

        if let Some(node) = devices
            .outputs
            .into_iter()
            .find(|d| !before.contains(&d.id) && is_own_stream_node(d))
        {
            return Ok((stream, Some(node)));
        }
    }

    eprintln!("Timed out waiting for the new PipeWire node, routing will be skipped");
    Ok((stream, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_gain() {
        assert_eq!(effective_gain(1.0, 1.0, 1.0), 1.0);
        assert_eq!(effective_gain(0.5, 0.5, 1.0), 0.25);

        // Amplification beyond 1.0 is allowed on purpose.
        assert_eq!(effective_gain(2.0, 1.0, 1.0), 2.0);
        assert_eq!(effective_gain(2.0, 1.0, 3.0), 6.0);

        // Anything that is not a usable gain collapses to silence.
        assert_eq!(effective_gain(0.0, 1.0, 1.0), 0.0);
        assert_eq!(effective_gain(-1.0, 1.0, 1.0), 0.0);
        assert_eq!(effective_gain(f32::NAN, 1.0, 1.0), 0.0);
        assert_eq!(effective_gain(f32::INFINITY, 1.0, 1.0), 0.0);
    }

    fn stream_node(name: &str, with_ports: bool) -> AudioDevice {
        use crate::types::pipewire::Port;

        let mut device = AudioDevice::new(1, None, None, Some(name), DeviceType::Output);
        if with_ports {
            device.add_port(Port {
                node_id: 1,
                port_id: 1,
                name: "output_FL".to_string(),
            });
            device.add_port(Port {
                node_id: 1,
                port_id: 2,
                name: "output_FR".to_string(),
            });
        }
        device
    }

    #[test]
    fn test_is_own_stream_node() {
        assert!(is_own_stream_node(&stream_node(
            "alsa_playback.pwsp-daemon",
            true
        )));
        assert!(is_own_stream_node(&stream_node("PWSP-daemon", true)));

        // Somebody else's playback stream must never be treated as ours.
        assert!(!is_own_stream_node(&stream_node("Firefox", true)));
        // Nor a node whose ports have not been discovered yet.
        assert!(!is_own_stream_node(&stream_node(
            "alsa_playback.pwsp-daemon",
            false
        )));
    }
}
