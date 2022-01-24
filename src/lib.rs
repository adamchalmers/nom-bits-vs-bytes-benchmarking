pub mod bitlevel;
pub mod bytelevel;

/// Newtype around a very common type in Nom.
/// Represents a binary sequence which can be parsed one bit at a time.
/// Nom represents this as a sequence of bytes, and an offset tracking which number bit
/// is currently being read.
///
/// For example, you might start with 16 bits, pointing at the 0th bit:
///```norun
/// 1111000011001100
/// ^
/// ```norun
/// Nom represents this using the BitInput type as:
/// ```
/// ([0b11110000, 0b11001100], 0)
///     ^
/// ```norun
/// Lets say you parsed 3 bits from there. After that, the BitInput would be
///
/// ```norun
/// ([0b11110000, 0b11001100], 3)
///        ^
/// ```norun
/// After reading another six bits, the input would have advanced past the first byte:
///
/// ```norun
/// ([0b11110000, 0b11001100], 9)
///                  ^
/// ```
/// Because the first byte will never be used again, Nom optimizes by dropping the first byte
///
/// ```norun
///  ([0b11001100], 1)
///       ^
/// ```
pub type BitInput<'a> = (&'a [u8], usize);

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
