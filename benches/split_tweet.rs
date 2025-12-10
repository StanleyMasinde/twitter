use std::fs;

use criterion::{BenchmarkId, criterion_main};
use criterion::{Criterion, criterion_group};
use twitter::twitter::tweet::Tweet;

fn criterion_benchmark(c: &mut Criterion) {
    let new_tweet = Tweet::default();
    let tweet = fs::read_to_string("./benches/thread.txt").unwrap();
    c.bench_with_input(BenchmarkId::new("split_tweet", &tweet), &tweet, |b, s| {
        b.iter(|| new_tweet.split_tweet(&s, "---"));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
