use crate::{from_nibble, Header, Packet};
use nom::{bytes::complete::take, IResult};

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
        _ => unimplemented!(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_packet() {
        let expected = Packet::Literal {
            version: 6,
            value: 2021,
        };
        let actual = parse("110100101111111000101000").unwrap().1;
        assert_eq!(actual, expected);
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
