use algc_codec::codec::{encode_decode_long_string, load_test_data};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

pub fn bench_codec_long_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_codec_long_string");
    group
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5))
        .sample_size(50);
    let file_contents = load_test_data();
    for (pos, input_string) in file_contents.iter().enumerate() {
        group.throughput(Throughput::Elements(input_string.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("encode_decode_long_string-{}", pos), ""),
            input_string,
            |b, data| b.iter(|| encode_decode_long_string(String::from(data), black_box(20))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_codec_long_string);
criterion_main!(benches);
