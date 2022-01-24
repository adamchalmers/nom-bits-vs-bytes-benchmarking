use crate::{from_nibble, Header, Operation, Packet};
use nom::{
    bytes::complete::take,
    combinator::map_res,
    multi::{length_count, length_value, many1},
    IResult,
};

impl Header {
    fn parse_bytes(i: &str) -> IResult<&str, Self> {
        let (i, version) = take(3usize)(i)?;
        let (i, type_id) = take(3usize)(i)?;
        let to_num = |binary| u8::from_str_radix(binary, 2).unwrap();
        Ok((
            i,
            Self {
                version: to_num(version),
                type_id: to_num(type_id),
            },
        ))
    }
}

/// Parse a Packet from a sequence of bytes.
pub fn parse(i: &str) -> IResult<&str, Packet> {
    let (i, header) = Header::parse_bytes(i)?;
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

/// Parse the number literal from a sequence of bytes.
fn parse_literal_number(mut i: &str) -> IResult<&str, u64> {
    let mut half_bytes = Vec::new();
    loop {
        let (remaining_i, continue_bit) = take(1usize)(i)?;
        i = remaining_i;
        let (remaining_i, four_bits) = take(4usize)(i)?;
        i = remaining_i;
        let nibble = u8::from_str_radix(four_bits, 2).expect("should be binary");
        half_bytes.push(nibble);
        if continue_bit == "0" {
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

/// Returns a parser which consumes the given number of binary characters from the start of the
/// string, then interprets it as a binary number and returns it as a `usize`.
fn parse_num(n: usize) -> impl FnMut(&str) -> IResult<&str, usize> {
    move |i| map_res(take(n), |s| usize::from_str_radix(s, 2))(i)
}

/// Parse a PacketBody::Operator from a sequence of bits.
fn parse_operator(i: &str, type_id: u8) -> IResult<&str, (Vec<Packet>, Operation)> {
    let (i, length_type_id) = take(1usize)(i)?;
    let (i, subpackets) = if length_type_id == "0" {
        // the next 15 bits are a number that represents
        // the total length in bits of the sub-packets contained by this packet.
        length_value(parse_num(15), many1(parse))(i)?
    } else {
        // the next 11 bits are a number that represents
        // the number of sub-packets immediately contained by this packet.
        length_count(parse_num(11), parse)(i)?
    };

    Ok((i, (subpackets, Operation::from(type_id))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal_packet() {
        let expected = Packet::Literal {
            version: 6,
            value: 2021,
        };
        let actual = parse("110100101111111000101000").unwrap().1;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_recursive_packet() {
        let expected = Packet::Operator {
            version: 1,
            type_id: Operation::Less,
            subpackets: vec![
                Packet::Literal {
                    version: 6,
                    value: 10,
                },
                Packet::Literal {
                    version: 2,
                    value: 20,
                },
            ],
        };
        let actual = parse("00111000000000000110111101000101001010010001001000000000")
            .unwrap()
            .1;
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_parse_header() {
        let expected = Header {
            version: 6,
            type_id: 4,
        };
        let actual = Header::parse_bytes("110100101111111000101000").unwrap().1;
        assert_eq!(actual, expected);
    }
}
