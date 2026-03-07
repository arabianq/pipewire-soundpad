use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

// A mock of AudioPlayer to isolate the play method's blocking behavior.
// We only implement the relevant part of the logic that needs optimizing.
pub struct AudioPlayerMock {
    pub tracks: std::collections::HashMap<u32, ()>,
    pub next_id: u32,
    pub volume: f32,
}

impl AudioPlayerMock {
    pub fn new() -> Self {
        AudioPlayerMock {
            tracks: std::collections::HashMap::new(),
            next_id: 1,
            volume: 1.0,
        }
    }

    pub async fn play(
        &mut self,
        file_path: &Path,
        concurrent: bool,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        if !file_path.exists() {
            return Err(format!("File does not exist: {}", file_path.display()).into());
        }

        let path_buf = file_path.to_path_buf();
        let _file = tokio::task::spawn_blocking(move || {
            // Simulate some blocking work like Decoder::try_from which reads file headers
            let _f = fs::File::open(&path_buf).unwrap();

            // Emulate the actual time spent reading file and decoding header (which is what Decoder::try_from does)
            std::thread::sleep(std::time::Duration::from_millis(100)); // Simulate slow disk/decode
            _f
        })
        .await?;

        if !concurrent {
            self.tracks.clear();
        }

        let id = self.next_id;
        self.next_id += 1;
        self.tracks.insert(id, ());

        Ok(id)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_performance_blocking() {
    println!("Setting up mock environment...");

    // Create a dummy file to read
    let test_file = Path::new("test_dummy.wav");
    fs::write(test_file, "dummy content").unwrap();

    let player = Arc::new(Mutex::new(AudioPlayerMock::new()));

    println!("Starting benchmark for synchronous behavior in async fn...");

    // We launch a background task that measures event loop latency.
    // If the main tasks block the executor, this task will suffer high latency.
    let latency_task = tokio::spawn(async {
        let mut max_latency = std::time::Duration::from_secs(0);
        for _ in 0..50 {
            let start = Instant::now();
            tokio::task::yield_now().await;
            let elapsed = start.elapsed();
            if elapsed > max_latency {
                max_latency = elapsed;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        max_latency
    });

    // Launch multiple play operations
    let mut tasks = vec![];
    let start_time = Instant::now();
    for _ in 0..10 {
        let player_clone = Arc::clone(&player);
        let file_path = test_file.to_path_buf();
        tasks.push(tokio::spawn(async move {
            let mut p = player_clone.lock().await;
            let _ = p.play(&file_path, true).await;
        }));
    }

    // Wait for all tasks to finish
    for t in tasks {
        let _ = t.await;
    }
    let total_time = start_time.elapsed();

    let max_latency = latency_task.await.unwrap();

    println!("Total execution time: {:?}", total_time);
    println!(
        "Max event loop latency (blocking indicator): {:?}",
        max_latency
    );

    // Cleanup
    fs::remove_file(test_file).unwrap();
}
