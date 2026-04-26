use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

#[derive(Clone)]
pub struct TrackInfo {
    pub id: u32,
    pub path: PathBuf,
    pub duration: Option<f32>,
    pub position: f32,
    pub volume: f32,
    pub looped: bool,
    pub paused: bool,
}

fn clone_tracks_benchmark(c: &mut Criterion) {
    let mut tracks = Vec::new();
    for i in 0..100 {
        tracks.push(TrackInfo {
            id: i,
            path: PathBuf::from(format!("/tmp/track_{}.mp3", i)),
            duration: Some(100.0),
            position: 0.0,
            volume: 1.0,
            looped: false,
            paused: false,
        });
    }

    c.bench_function("clone_100_tracks", |b| b.iter(|| {
        let cloned = black_box(&tracks).clone();
        for track in cloned {
            black_box(track.id);
        }
    }));

    c.bench_function("iter_100_tracks", |b| b.iter(|| {
        for track in black_box(&tracks).iter() {
            black_box(track.id);
        }
    }));
}

criterion_group!(benches, clone_tracks_benchmark);
criterion_main!(benches);
