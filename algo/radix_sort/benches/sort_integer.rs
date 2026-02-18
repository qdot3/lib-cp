use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{rng, Rng};
use std::{
    fs::File,
    io::{Read, Write},
    u64,
};

use radix_sort::RadixSort;

fn base(c: &mut Criterion, group_name: &str, mut gen_input: impl FnMut(usize) -> Vec<u64>) {
    let mut group = c.benchmark_group(group_name);

    for size in (12..=20).step_by(4).map(|n| 1 << n) {
        let path = format!(
            "./benches/dataset/{}_{}.txt",
            group_name
                .replace(|c: char| !c.is_ascii_alphabetic(), "_")
                .to_ascii_lowercase(),
            size
        );
        let input = if let Ok(mut f) = File::open(&path) {
            let mut buf = String::new();
            f.read_to_string(&mut buf).unwrap();

            buf.split_ascii_whitespace()
                .filter_map(|n| u64::from_str_radix(n, 10).ok())
                .collect()
        } else {
            let input = gen_input(size);

            let mut f = File::create(&path).unwrap();
            for i in input.iter() {
                f.write(i.to_string().as_bytes()).unwrap();
                f.write(b" ").unwrap();
            }

            input
        };

        group.bench_function(BenchmarkId::new("radix sort", size), |b| {
            b.iter_batched_ref(
                || input.clone(),
                |input| input.as_mut_slice().radix_sort(),
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("std::slice::sort_unstable", size), |b| {
            b.iter_batched_ref(
                || input.clone(),
                |input| input.sort_unstable(),
                BatchSize::SmallInput,
            );
        });
    }
}

fn cmp_random(c: &mut Criterion) {
    base(c, "Random", |size| rng().random_iter().take(size).collect());
}

/// 基数ソートの最悪ケースかつ`sort_unstable()`のベターケース（k = 1000）
fn cmp_random_large(c: &mut Criterion) {
    base(c, "Random (Large)", |size| {
        let mut rng = rng();
        std::iter::repeat_with(|| rng.random_range(u64::MAX - 1000..u64::MAX))
            .take(size)
            .collect()
    })
}

criterion_group!(
    benches,
    cmp_random,
    cmp_random_large
);
criterion_main!(benches);
