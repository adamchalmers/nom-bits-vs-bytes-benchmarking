use crate::{Header, Operation, Packet};
use nom::{bits::complete::take, multi::length_count, IResult};

/// Takes n bits from the BitInput.
/// Returns the remaining BitInput and a number parsed the first n bits.
fn take_up_to_16_bits(i: BitInput, n: u8) -> IResult<BitInput, u16> {
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
        other => parse_operator(i, other).map(|(i, (subpackets, type_id))| {
            (
                i,
                Packet::Operator {
                    version: header.version,
                    subpackets,
                    type_id,
                },
            )
        }),
    }
}

/// Parse a PacketBody::Operator from a sequence of bits.
fn parse_operator(i: BitInput, type_id: u8) -> IResult<BitInput, (Vec<Packet>, Operation)> {
    let (i, length_type_id) = take_up_to_16_bits(i, 1)?;
    let (i, subpackets) = if length_type_id == 0 {
        // the next 15 bits are a number that represents
        // the total length in bits of the sub-packets contained by this packet.
        let (mut i, total_subpacket_lengths) = take_up_to_16_bits(i, 15)?;

        let mut subpackets = Vec::new();
        let initial_bits_remaining = bits_remaining(&i);
        // Parse subpackets until the correct number of bits have been read.
        while initial_bits_remaining - bits_remaining(&i) < (total_subpacket_lengths as usize) {
            let (remaining_i, packet) = parse(i)?;
            i = remaining_i;
            subpackets.push(packet);
        }
        (i, subpackets)
    } else {
        // the next 11 bits are a number that represents
        // the number of sub-packets immediately contained by this packet.
        let parse_num_subpackets = |i| take_up_to_16_bits(i, 11);
        length_count(parse_num_subpackets, parse)(i)?
    };

    Ok((i, (subpackets, Operation::from(type_id))))
}

impl Header {
    fn parse_bits(i: BitInput) -> IResult<BitInput, Self> {
        let (i, version) = take_up_to_16_bits(i, 3)?;
        let (i, type_id) = take_up_to_16_bits(i, 3)?;
        Ok((
            i,
            Self {
                version: version.try_into().unwrap(),
                type_id: type_id.try_into().unwrap(),
            },
        ))
    }
}

/// Parse the number literal from a sequence of bits.
fn parse_literal_number(mut i: BitInput) -> IResult<BitInput, u64> {
    let mut half_bytes = Vec::new();
    loop {
        let (remaining_i, bit) = take_up_to_16_bits(i, 1)?;
        let (remaining_i, half_byte) = take_up_to_16_bits(remaining_i, 4)?;
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
        .map(|(i, nibble)| (nibble as u64) << (4 * i))
        .sum();
    Ok((i, num))
}

/// Newtype around a very common type in Nom.
/// Represents a binary sequence which can be parsed one bit at a time.
/// Nom represents this as a sequence of bytes, and an offset tracking which number bit
/// is currently being read.
///
/// For example, you might start with 16 bits, pointing at the 0th bit:
///```text
/// 1111000011001100
/// ^
/// ```text
/// Nom represents this using the BitInput type as:
/// ```
/// ([0b11110000, 0b11001100], 0)
///     ^
/// ```text
/// Lets say you parsed 3 bits from there. After that, the BitInput would be
///
/// ```text
/// ([0b11110000, 0b11001100], 3)
///        ^
/// ```text
/// After reading another six bits, the input would have advanced past the first byte:
///
/// ```text
/// ([0b11110000, 0b11001100], 9)
///                  ^
/// ```
/// Because the first byte will never be used again, Nom optimizes by dropping the first byte
///
/// ```text
///  ([0b11001100], 1)
///       ^
/// ```
pub type BitInput<'a> = (&'a [u8], usize);

/// How many bits can still be parsed from the BitInput.
fn bits_remaining(i: &BitInput) -> usize {
    // How far through the first byte are we?
    let bits_in_first_byte = 8 - i.1;
    // And how many bytes are left after that?
    let remaining_bytes = i.0.len() - 1;
    bits_in_first_byte + (8 * remaining_bytes)
}

#[cfg(test)]
mod tests {
    use crate::bitlevel::BitInput;

    #[test]
    fn test_tag() {
        let tests: Vec<(Vec<u8>, u8, u8, bool)> = vec![
            (vec![0xff], 4, 0x0f, true),
            (vec![0xff], 1, 0x01, true),
            (vec![0xff], 2, 0x01, false),
            (vec![0xff], 8, 0xfe, false),
        ];
        fn parser(
            pattern: u8,
            num_bits_to_compare: u8,
            input: BitInput,
        ) -> nom::IResult<BitInput, u8> {
            nom::bits::complete::tag(pattern, num_bits_to_compare)(input)
        }
        for (test_num, (input, num_bits_to_compare, pattern, expected)) in
            tests.into_iter().enumerate()
        {
            let answer = parser(pattern, num_bits_to_compare, (input.as_slice(), 0));
            assert_eq!(answer.is_ok(), expected, "Failed test #0{test_num}");
        }
    }

    #[test]
    fn test_take_bit() {
        use nom::{bits::complete::take, IResult};
        type BitInput<'a> = (&'a [u8], usize);
        pub fn take_bit(i: BitInput) -> IResult<BitInput, bool> {
            let (i, bit): (BitInput, u8) = take(1u8)(i)?;
            Ok((i, bit != 0))
        }

        let input = ([0b10101010].as_ref(), 0);
        let (input, first_bit) = take_bit(input).unwrap();
        assert!(first_bit); // First bit is 1
        let (_input, second_bit) = take_bit(input).unwrap();
        assert!(!second_bit); // Second bit is 0
    }

    #[test]
    fn test_take_nibble() {
        use nom::{bits::complete::take, IResult};
        type BitInput<'a> = (&'a [u8], usize);

        /// Take 4 bits from the BitInput.
        /// Store the output in a u8, because there's no u4 type, and u8 is the closest-available size.
        pub fn take_nibble(i: BitInput) -> IResult<BitInput, u8> {
            take(4usize)(i)
        }

        let input = ([0b1010_1111].as_ref(), 0);
        let (_input, actual_nibble) = take_nibble(input).unwrap();
        let expected_nibble = 0b1010;
        assert_eq!(actual_nibble, expected_nibble);
    }
}
