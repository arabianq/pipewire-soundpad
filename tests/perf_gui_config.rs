use pwsp::types::config::GuiConfig;
use std::time::Instant;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_gui_config_save_performance() {
    println!("Setting up mock environment for GUI config save...");

    // Create a dummy config to write a lot of data
    let mut dirs = vec![];
    for i in 0..10000 {
        dirs.push(std::path::PathBuf::from(format!("/tmp/dir_{}", i)));
    }

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

    let start_time = Instant::now();
    let mut tasks = vec![];
    for _ in 0..100 {
        let dirs_clone = dirs.clone();
        tasks.push(tokio::spawn(async move {
            let mut config = GuiConfig::default();
            config.dirs = dirs_clone;

            // Since `save_to_file` does IO, if it is synchronous, it will block this thread
            let _ = config.save_to_file();
        }));
    }

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
}
