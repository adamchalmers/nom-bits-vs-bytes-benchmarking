use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nom_bits::*;
/// Benchmark the `set_all_parallel` method of Gridlike.
fn benchmark_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parser");
    let inputs = [
        (
            "Literal",
            vec![0b11010010, 0b11111110, 0b00101000],
            "110100101111111000101000",
        ),
        (
            "Two subpackets",
            vec![
                0b00111000, 0b00000000, 0b01101111, 0b01000101, 0b00101001, 0b00010010, 0b00000000,
            ],
            "00111000000000000110111101000101001010010001001000000000",
        ),
        (
            "Three subpackets",
            vec![
                0b11101110, 0b00000000, 0b11010100, 0b00001100, 0b10000010, 0b00110000, 0b01100000,
            ],
            "11101110000000001101010000001100100000100011000001100000",
        ),
    ];
    for (name, bit_input, byte_input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("Bit parser", name),
            &bit_input,
            |b, bit_input| {
                b.iter(|| black_box(bitparser(bit_input)).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Byte parser", name),
            &byte_input,
            |b, byte_input| {
                b.iter(|| black_box(bytelevel::parse(byte_input)).unwrap());
            },
        );
    }
    group.finish();
}

pub fn bitparser(i: &[u8]) -> nom::IResult<&[u8], Packet> {
    nom::bits::bits(bitlevel::parse)(i)
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
