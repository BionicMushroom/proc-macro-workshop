// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.

//! Provides the [`#[bitfield]`](macro@bitfield) attribute macro and
//! the [`BitfieldSpecifier`](macro@BitfieldSpecifier) derive macro.

/// An attribute macro that provides a mechanism for defining structs in a packed binary
/// representation with access to ranges of bits, similar to the language-level
/// support for [bit fields in C](https://en.cppreference.com/w/cpp/language/bit_field).
///
/// The macro will conceptualize one of these structs as a sequence of bits 0..N.
/// The bits are grouped into fields in the order specified by a struct written by
/// the caller. The [`#[bitfield]`](macro@bitfield) attribute rewrites the caller's struct into a
/// private byte array representation with public getter and setter methods for each
/// field.
///
/// The total number of bits N is required to be a multiple of 8 (this will be
/// checked at compile time).
///
/// For example, the following invocation builds a struct with a total size of 32
/// bits or 4 bytes. It places field `a` in the least significant bit of the first
/// byte, field `b` in the next three least significant bits, field `c` in the
/// remaining four most significant bits of the first byte, and field `d` spanning
/// the next three bytes.
///
/// ```
/// use bitfield::{bitfield, B1, B24, B3, B4};
///
/// #[bitfield]
/// pub struct MyFourBytes {
///     a: B1,
///     b: B3,
///     c: B4,
///     d: B24,
/// }
/// ```
///
/// ```text
///                                least significant bit of third byte
///                                  ┊           most significant
///                                  ┊             ┊
///                                  ┊             ┊
/// ║  first byte   ║  second byte  ║  third byte   ║  fourth byte  ║
/// ╟───────────────╫───────────────╫───────────────╫───────────────╢
/// ║▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒║
/// ╟─╫─────╫───────╫───────────────────────────────────────────────╢
/// ║a║  b  ║   c   ║                       d                       ║
///                  ┊                                             ┊
///                  ┊                                             ┊
///                least significant bit of d         most significant
/// ```
///
/// The code emitted by the [`#[bitfield]`](macro@bitfield) macro for this struct would be as follows.
/// Note that the field getters and setters use whichever of [`u8`](https://doc.rust-lang.org/std/primitive.u8.html),
/// [`u16`](https://doc.rust-lang.org/std/primitive.u16.html), [`u32`](https://doc.rust-lang.org/std/primitive.u32.html),
/// [`u64`](https://doc.rust-lang.org/std/primitive.u64.html) or [`u128`](https://doc.rust-lang.org/std/primitive.u128.html)
/// is the smallest while being at least as large as the number of bits in the field.
///
/// ```ignore
/// impl MyFourBytes {
///     // Initializes all fields to 0.
///     fn new() -> Self;
///
///     // Field getters and setters:
///     fn get_a(&self) -> u8;
///     fn set_a(&mut self, val: u8);
///     fn get_b(&self) -> u8;
///     fn set_b(&mut self, val: u8);
///     fn get_c(&self) -> u8;
///     fn set_c(&mut self, val: u8);
///     fn get_d(&self) -> u32;
///     fn set_d(&mut self, val: u32);
/// }
/// ```
///
/// The macro can also be applied to a struct with unnamed fields:
///
/// ```
/// use bitfield::{bitfield, B1, B24, B3, B4};
///
/// #[bitfield]
/// pub struct MyFourBytes(B1, B3, B4, B24);
/// ```
///
/// The generated code for this struct would be as follows:
///
/// ```ignore
/// impl MyFourBytes {
///     fn get_0(&self) -> u8;
///     fn set_0(&mut self, val: u8);
///     fn get_1(&self) -> u8;
///     fn set_1(&mut self, val: u8);
///     fn get_2(&self) -> u8;
///     fn set_2(&mut self, val: u8);
///     fn get_3(&self) -> u32;
///     fn set_3(&mut self, val: u32);
/// }
/// ```
///
/// Using the [`BitfieldSpecifier`](macro@BitfieldSpecifier) derive macro,
/// you can also mark an enum as a bitfield specifier and then use that enum
/// as a field type in a bitfield struct. [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)
/// is also supported as a bitfield specifier:
///
/// ```
/// use bitfield::{bitfield, BitfieldSpecifier, B1, B3};
///
/// #[bitfield]
/// pub struct RedirectionTableEntry {
///     acknowledged: bool,
///     trigger_mode: TriggerMode,
///     delivery_mode: DeliveryMode,
///     reserved: B3,
/// }
///
/// #[derive(BitfieldSpecifier)]
/// pub enum TriggerMode {
///     Edge = 0,
///     Level = 1,
/// }
///
/// #[derive(BitfieldSpecifier)]
/// pub enum DeliveryMode {
///     Fixed = 0b000,
///     Lowest = 0b001,
///     SMI = 0b010,
///     RemoteRead = 0b011,
///     NMI = 0b100,
///     Init = 0b101,
///     Startup = 0b110,
///     External = 0b111,
/// }
/// ```
///
/// The generated code for this struct would be as follows:
///
/// ```ignore
/// impl RedirectionTableEntry {
///     fn new() -> Self;
///     fn get_acknowledged(&self) -> bool;
///     fn set_acknowledged(&mut self, val: bool);
///     fn get_trigger_mode(&self) -> TriggerMode;
///     fn set_trigger_mode(&mut self, val: TriggerMode);
///     fn get_delivery_mode(&self) -> DeliveryMode;
///     fn set_delivery_mode(&mut self, val: DeliveryMode);
///     fn get_reserved(&self) -> u8;
///     fn set_reserved(&mut self, val: u8);
/// }
/// ```
///
/// For documentation purposes, the `#[bits = N]` attribute is also provided.
/// It can be used to document the size, in bits, of a struct, struct field or enum.
/// The item on which the attribute is applied will fail to compile if
/// its size does not match the value of the attribute. Here is the preceding example
/// with the `#[bits = N]` attribute applied:
///
/// ```
/// use bitfield::{bitfield, BitfieldSpecifier, B1, B3};
///
/// #[bitfield]
/// #[bits = 8]
/// pub struct RedirectionTableEntry {
///     #[bits = 1]
///     acknowledged: bool,
///     #[bits = 1]
///     trigger_mode: TriggerMode,
///     #[bits = 3]
///     delivery_mode: DeliveryMode,
///     #[bits = 3]
///     reserved: B3,
/// }
///
/// #[derive(BitfieldSpecifier)]
/// #[bits = 1]
/// pub enum TriggerMode {
///     Edge = 0,
///     Level = 1,
/// }
///
/// #[derive(BitfieldSpecifier)]
/// #[bits = 3]
/// pub enum DeliveryMode {
///     Fixed = 0b000,
///     Lowest = 0b001,
///     SMI = 0b010,
///     RemoteRead = 0b011,
///     NMI = 0b100,
///     Init = 0b101,
///     Startup = 0b110,
///     External = 0b111,
/// }
/// ```
pub use bitfield_impl::bitfield;

/// A derive macro that allows an enum to be used as a bitfield specifier
/// (a field in a struct marked with the [`#[bitfield]`](macro@bitfield) attribute).
/// The enum must have a power-of-two number of variants or be marked with the
/// `#[set_bits = N]` attribute, otherwise it will fail to compile.
///
/// For example, here is an enum with a power-of-two number of variants and
/// its use inside a bitfield struct:
///
/// ```
/// use bitfield::{bitfield, BitfieldSpecifier, B3, B5};
///
/// #[derive(BitfieldSpecifier)]
/// pub enum DeliveryMode {
///     Fixed = 0b000,
///     Lowest = 0b001,
///     SMI = 0b010,
///     RemoteRead = 0b011,
///     NMI = 0b100,
///     Init = 0b101,
///     Startup = 0b110,
///     External = 0b111,
/// }
///
/// #[bitfield]
/// pub struct RedirectionTableEntry {
///     delivery_mode: DeliveryMode,
///     reserved: B5,
/// }
/// ```
///
/// The generated code for this bitfield struct would be as follows:
///
/// ```ignore
/// impl RedirectionTableEntry {
///     fn new() -> Self;
///     fn get_delivery_mode(&self) -> DeliveryMode;
///     fn set_delivery_mode(&mut self, val: DeliveryMode);
///     fn get_reserved(&self) -> u8;
///     fn set_reserved(&mut self, val: u8);
/// }
/// ```
///
/// Here is an enum marked with the `#[set_bits = N]` attribute.
/// By specifying the `#[set_bits = N]` attribute, the enum will be treated as
/// a bitfield specifier with `N` bits:
///
/// ```
/// use bitfield::{bitfield, BitfieldSpecifier, B3, B5};
///
/// #[derive(BitfieldSpecifier)]
/// #[set_bits = 3]
/// pub enum DeliveryMode {
///     SMI = 0b010,
///     NMI = 0b100,
/// }
///
/// #[bitfield]
/// pub struct RedirectionTableEntry {
///     delivery_mode: DeliveryMode,
///     reserved: B5,
/// }
/// ```
///
/// Because the enum marked with the `#[set_bits = N]` attribute might not exhaustively cover
/// the range of bits, the getter will return a [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// where the [`Ok`](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok) contains the enum variant
/// in case a variant matches the bit pattern, and the [`Err`](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err)
/// contains an [`Unrecognized`] type that can be used to access the raw value of the bit pattern.
/// As such, the generated code for the bitfield struct in the example above would be as follows:
///
/// ```ignore
/// impl RedirectionTableEntry {
///     fn new() -> Self;
///     fn get_delivery_mode(&self) -> Result<DeliveryMode, Unrecognized<u8>>;
///     fn set_delivery_mode(&mut self, val: DeliveryMode);
///     fn get_reserved(&self) -> u8;
///     fn set_reserved(&mut self, val: u8);
/// }
/// ```
///
/// For documentation purposes, the `#[bits = N]` attribute can also be applied to the enum
/// (as well as on struct and struct fields) to document the item's size, in bits. If the
/// size does not match the value of the attribute, the program will fail to compile.
/// Here is the enum from the first example with the `#[bits = N]` attribute applied:
///
/// ```
/// use bitfield::{BitfieldSpecifier, B3};
///
/// #[derive(BitfieldSpecifier)]
/// #[bits = 3]
/// pub enum DeliveryMode {
///     Fixed = 0b000,
///     Lowest = 0b001,
///     SMI = 0b010,
///     RemoteRead = 0b011,
///     NMI = 0b100,
///     Init = 0b101,
///     Startup = 0b110,
///     External = 0b111,
/// }
/// ```
pub use bitfield_impl::BitfieldSpecifier;

/// The error type returned when a bitfield specifier cannot be converted
/// from a bit pattern because no matching variant exists.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Unrecognized<T> {
    value: T,
}

impl<T: Copy> Unrecognized<T> {
    /// Returns the raw value of the bit pattern that could not be converted.
    pub fn raw_value(&self) -> T {
        self.value
    }

    #[doc(hidden)]
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

use std::num::NonZero;

/// A trait implemented by all bitfield specifiers.
pub trait Specifier {
    #[doc(hidden)]
    const BITS: NonZero<u8>;

    /// The type returned by bitfield getters for this bitfield specifier.
    type GetType;

    /// The type returned by bitfield setters for this bitfield specifier.
    type SetType;

    #[doc(hidden)]
    #[must_use]
    fn get(data_storage: &[u8], bit_index: usize) -> Self::GetType;

    #[doc(hidden)]
    fn set(data_storage: &mut [u8], bit_index: usize, value: Self::SetType);
}

#[doc(hidden)]
impl Specifier for bool {
    const BITS: NonZero<u8> = NonZero::new(1).expect("1 must be greater than 0");

    type GetType = bool;
    type SetType = bool;

    fn get(data_storage: &[u8], bit_index: usize) -> Self::GetType {
        let byte_offset = bit_index / 8;
        let bit_offset = bit_index % 8;

        let data_byte = &data_storage[byte_offset];
        let bit = (data_byte >> bit_offset) & 1;

        bit != 0
    }

    fn set(data_storage: &mut [u8], bit_index: usize, value: Self::SetType) {
        let byte_offset = bit_index / 8;
        let bit_offset = u8::try_from(bit_index % 8).expect("bit_offset should be in range for u8");

        let data_byte = &mut data_storage[byte_offset];
        let mask = !(1u8 << bit_offset);

        *data_byte &= mask;
        *data_byte |= u8::from(value) << bit_offset;
    }
}

bitfield_impl::generate_bit_specifiers!();
