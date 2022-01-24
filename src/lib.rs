pub mod bitlevel;
pub mod bytelevel;

/// A tree structure that represents some number. Can be parsed out of its binary encoding.
#[derive(Eq, PartialEq, Debug)]
pub enum Packet {
    /// Leaf node.
    /// Represents a number directly.
    Literal { version: u8, value: u64 },
    /// Internal node.
    /// Represents the number you get from running the given operation on the given subpackets.
    Operator {
        version: u8,
        type_id: Operation,
        subpackets: Vec<Packet>,
    },
}

/// Each operator packet has an operation which it runs on the values of its subpackets.
#[derive(Eq, PartialEq, Debug)]
pub enum Operation {
    Sum,
    Product,
    Min,
    Max,
    Greater,
    Less,
    Equal,
}

/// Every type_id corresponds to a particular operation.
impl From<u8> for Operation {
    fn from(type_id: u8) -> Self {
        match type_id {
            0 => Self::Sum,
            1 => Self::Product,
            2 => Self::Min,
            3 => Self::Max,
            4 => panic!("Literals should not be parsed into Operations"),
            5 => Self::Greater,
            6 => Self::Less,
            7 => Self::Equal,
            other => panic!("illegal type_id {}", other),
        }
    }
}
/// Every packet has a header.
#[derive(Eq, PartialEq, Debug)]
pub struct Header {
    version: u8,
    type_id: u8,
}

/// A nibble is a u4 (half a byte). But Rust doesn't have a u4 type!
/// So we store the u4s in u8s, and then use bit-shifting operations to put them into the right
/// column of the larger binary number we're working with.
pub fn from_nibble((i, nibble): (usize, u8)) -> u64 {
    (nibble as u64) << (4 * i)
}
