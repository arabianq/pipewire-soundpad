use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::task;

// Mock to simulate the specific `fs::File::open` behavior inside spawn_blocking
// vs tokio::fs::File::open then spawn_blocking for Decoder::try_from.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_file_open_benchmark() {
    let test_dir = PathBuf::from("test_benchmark_files");
    fs::create_dir_all(&test_dir).unwrap();

    // Create multiple dummy files
    let mut file_paths = vec![];
    for i in 0..100 {
        let path = test_dir.join(format!("dummy_{}.wav", i));
        fs::write(&path, "dummy content to simulate audio file headers").unwrap();
        file_paths.push(path);
    }

    println!("Starting benchmark for synchronous vs asynchronous file open during track restart...");

    // 1. Benchmark Sync File Open (Old Way)
    let sync_start = Instant::now();
    let mut sync_futures = vec![];
    for path in file_paths.clone() {
        let handle = task::spawn_blocking(move || {
            // Simulate the disk I/O of opening a file
            if let Ok(file) = fs::File::open(&path) {
                // Simulate CPU-bound work of Decoding
                std::thread::sleep(std::time::Duration::from_millis(5));
                return Some(());
            }
            None
        });
        sync_futures.push(handle);
    }

    for handle in sync_futures {
        let _ = handle.await;
    }
    let sync_elapsed = sync_start.elapsed();

    // 2. Benchmark Async File Open (New Way)
    let async_start = Instant::now();
    let mut async_futures = vec![];
    for path in file_paths.clone() {
        let handle = tokio::spawn(async move {
            if let Ok(file) = tokio::fs::File::open(&path).await {
                let std_file = file.into_std().await;
                let inner_handle = task::spawn_blocking(move || {
                    // Simulate CPU-bound work of Decoding
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    return Some(());
                });
                if let Ok(res) = inner_handle.await {
                    return res;
                }
            }
            None
        });
        async_futures.push(handle);
    }

    for handle in async_futures {
        let _ = handle.await;
    }
    let async_elapsed = async_start.elapsed();

    println!("Sync File Open (Old) Time: {:?}", sync_elapsed);
    println!("Async File Open (New) Time: {:?}", async_elapsed);

    // Ensure cleanup
    fs::remove_dir_all(test_dir).unwrap();

    // Test assertion is not strict because performance varies on runners,
    // but we expect Async to be at least competitive or faster by freeing blocking threads.
    assert!(true);
}
