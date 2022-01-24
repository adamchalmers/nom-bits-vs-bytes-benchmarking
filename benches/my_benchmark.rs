use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nom_bits::*;
/// Benchmark the `set_all_parallel` method of Gridlike.
fn benchmark_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse Literal");
    let inputs = [(
        vec![0b11010010, 0b11111110, 0b00101000],
        "110100101111111000101000",
    )];
    for (test_num, (bit_input, byte_input)) in inputs.into_iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("Bit parser", test_num),
            &bit_input,
            |b, bit_input| {
                b.iter(|| black_box(bitlevel::run_parser(&bit_input)).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Byte parser", test_num),
            &byte_input,
            |b, byte_input| {
                b.iter(|| black_box(bytelevel::parse(byte_input)).unwrap());
            },
        );
    }
    group.finish();
}
criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
