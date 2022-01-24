use crate::{from_nibble, BitInput, Header, Packet};
use nom::{bits::complete::take, IResult};

/// Takes n bits from the BitInput.
/// Returns the remaining BitInput and a number parsed the first n bits.
fn take_up_to_8_bits(i: BitInput, n: u8) -> IResult<BitInput, u8> {
    take(n)(i)
}

/// Parse a Packet from a sequence of bits.
pub fn parse(i: BitInput) -> IResult<BitInput, Packet> {
    let (i, header) = Header::parse_bits(i)?;
    match header.type_id {
        4 => parse_literal_number(i).map(|(i, value)| {
            (
                i,
                Packet::Literal {
                    version: header.version,
                    value,
                },
            )
        }),
        _ => unimplemented!(),
    }
}

impl Header {
    fn parse_bits(i: BitInput) -> IResult<BitInput, Self> {
        let (i, version) = take_up_to_8_bits(i, 3)?;
        let (i, type_id) = take_up_to_8_bits(i, 3)?;
        Ok((i, Self { version, type_id }))
    }
}

/// Parse the number literal from a sequence of bits.
fn parse_literal_number(mut i: BitInput) -> IResult<BitInput, u64> {
    let mut half_bytes = Vec::new();
    loop {
        let (remaining_i, bit) = take_up_to_8_bits(i, 1)?;
        let (remaining_i, half_byte) = take_up_to_8_bits(remaining_i, 4)?;
        i = remaining_i;
        half_bytes.push(half_byte);
        if bit == 0 {
            break;
        }
    }
    let n = half_bytes.len() - 1;
    let num: u64 = half_bytes
        .into_iter()
        .enumerate()
        .map(|(i, b)| (n - i, b))
        .map(from_nibble)
        .sum();
    Ok((i, num))
}
