use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;

fn simulate_loop(children: &[PathBuf], search_query: &str) -> usize {
    let mut count = 0;
    for child in children {
        if !child.is_dir() {
            let ext = child
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let supported = [
                "mp3", "wav", "ogg", "flac", "mp4", "m4a", "aac", "mov", "mkv", "mka", "webm",
                "avi", "opus",
            ];
            if !supported.contains(&ext) {
                continue;
            }
            if !search_query.is_empty() {
                let file_name = child
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !file_name.to_lowercase().contains(search_query) {
                    continue;
                }
            }
        }
        count += 1;
    }
    count
}

fn simulate_optimized_loop(children: &[PathBuf], search_query: &str) -> usize {
    let mut count = 0;
    for child in children {
        if !child.is_dir()
            && !search_query.is_empty() {
                let file_name = child
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !file_name.to_lowercase().contains(search_query) {
                    continue;
                }
            }
        count += 1;
    }
    count
}

fn benchmark(c: &mut Criterion) {
    let mut children = Vec::new();
    for i in 0..10000 {
        let ext = if i % 2 == 0 { "mp3" } else { "txt" };
        children.push(PathBuf::from(format!("file_{}.{}", i, ext)));
    }
    let search_query = "";

    c.bench_function("unoptimized_loop", |b| {
        b.iter(|| simulate_loop(black_box(&children), black_box(search_query)))
    });

    let filtered_children: Vec<_> = children
        .into_iter()
        .filter(|child| {
            let ext = child
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let supported = [
                "mp3", "wav", "ogg", "flac", "mp4", "m4a", "aac", "mov", "mkv", "mka", "webm",
                "avi", "opus",
            ];
            supported.contains(&ext)
        })
        .collect();

    c.bench_function("optimized_loop", |b| {
        b.iter(|| simulate_optimized_loop(black_box(&filtered_children), black_box(search_query)))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
