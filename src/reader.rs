use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};
use borrow::Cow;

use endianity::Endianity;
use leb128;
use parser::{Error, Format, Result};

/// A trait for offsets with a DWARF section.
///
/// This allows consumers to choose a size that is appropriate for their address space.
pub trait ReaderOffset
    : Debug + Copy + Eq + Ord + Add<Output = Self> + AddAssign + Sub<Output = Self>
    {
    /// Convert a u8 to an offset.
    fn from_u8(offset: u8) -> Self;

    /// Convert a u16 to an offset.
    fn from_u16(offset: u16) -> Self;

    /// Convert an i16 to an offset.
    fn from_i16(offset: i16) -> Self;

    /// Convert a u32 to an offset.
    fn from_u32(offset: u32) -> Self;

    /// Convert a u64 to an offset.
    ///
    /// Returns `Error::UnsupportedOffset` if the value is too large.
    fn from_u64(offset: u64) -> Result<Self>;

    /// Convert an offset to a u64.
    fn into_u64(self) -> u64;

    /// Wrapping (modular) addition. Computes `self + other`.
    fn wrapping_add(self, other: Self) -> Self;

    /// Checked subtraction. Computes `self - other`.
    fn checked_sub(self, other: Self) -> Option<Self>;
}

impl ReaderOffset for u64 {
    #[inline]
    fn from_u8(offset: u8) -> Self {
        offset as u64
    }

    #[inline]
    fn from_u16(offset: u16) -> Self {
        offset as u64
    }

    #[inline]
    fn from_i16(offset: i16) -> Self {
        offset as u64
    }

    #[inline]
    fn from_u32(offset: u32) -> Self {
        offset as u64
    }

    #[inline]
    fn from_u64(offset: u64) -> Result<Self> {
        Ok(offset)
    }

    #[inline]
    fn into_u64(self) -> u64 {
        self
    }

    #[inline]
    fn wrapping_add(self, other: Self) -> Self {
        self.wrapping_add(other)
    }

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
}

impl ReaderOffset for u32 {
    #[inline]
    fn from_u8(offset: u8) -> Self {
        offset as u32
    }

    #[inline]
    fn from_u16(offset: u16) -> Self {
        offset as u32
    }

    #[inline]
    fn from_i16(offset: i16) -> Self {
        offset as u32
    }

    #[inline]
    fn from_u32(offset: u32) -> Self {
        offset
    }

    #[inline]
    fn from_u64(offset64: u64) -> Result<Self> {
        let offset = offset64 as u32;
        if offset as u64 == offset64 {
            Ok(offset)
        } else {
            Err(Error::UnsupportedOffset)
        }
    }

    #[inline]
    fn into_u64(self) -> u64 {
        self as u64
    }

    #[inline]
    fn wrapping_add(self, other: Self) -> Self {
        self.wrapping_add(other)
    }

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
}

impl ReaderOffset for usize {
    #[inline]
    fn from_u8(offset: u8) -> Self {
        offset as usize
    }

    #[inline]
    fn from_u16(offset: u16) -> Self {
        offset as usize
    }

    #[inline]
    fn from_i16(offset: i16) -> Self {
        offset as usize
    }

    #[inline]
    fn from_u32(offset: u32) -> Self {
        offset as usize
    }

    #[inline]
    fn from_u64(offset64: u64) -> Result<Self> {
        let offset = offset64 as usize;
        if offset as u64 == offset64 {
            Ok(offset)
        } else {
            Err(Error::UnsupportedOffset)
        }
    }

    #[inline]
    fn into_u64(self) -> u64 {
        self as u64
    }

    #[inline]
    fn wrapping_add(self, other: Self) -> Self {
        self.wrapping_add(other)
    }

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }
}

/// A trait for reading the data from a DWARF section.
///
/// All read operations advance the section offset of the reader
/// unless specified otherwise.
pub trait Reader: Debug + Clone + Send + Sync {
    /// The endianity of bytes that are read.
    type Endian: Endianity;

    /// The type used for offsets and lengths.
    type Offset: ReaderOffset;

    /// Return the endianity of bytes that are read.
    fn endian(&self) -> Self::Endian;

    /// Return the number of bytes remaining.
    fn len(&self) -> Self::Offset;

    /// Return true if the number of bytes remaining is zero.
    fn is_empty(&self) -> bool;

    /// Set the number of bytes remaining to zero.
    fn empty(&mut self);

    /// Set the number of bytes remaining to the specified length.
    fn truncate(&mut self, len: Self::Offset) -> Result<()>;

    /// Return the offset of this reader's data relative to the start of
    /// the given base reader's data.
    ///
    /// May panic if this reader's data is not contained within the given
    /// base reader's data.
    fn offset_from(&self, base: &Self) -> Self::Offset;

    /// Find the index of the first occurence of the given byte.
    /// The offset of the reader is not changed.
    fn find(&self, byte: u8) -> Result<Self::Offset>;

    /// Discard the specified number of bytes.
    fn skip(&mut self, len: Self::Offset) -> Result<()>;

    /// Split a reader in two.
    ///
    /// A new reader is returned that can be used to read the next
    /// `len` bytes, and `self` is advanced so that it reads the remainder.
    fn split(&mut self, len: Self::Offset) -> Result<Self>;

    /// Return all remaining data as a clone-on-write slice.
    ///
    /// The slice will be borrowed where possible, but some readers may
    /// always return an owned vector.
    ///
    /// Does not advance the reader.
    fn to_slice(&self) -> Result<Cow<[u8]>>;

    /// Convert all remaining data to a clone-on-write string.
    ///
    /// The string will be borrowed where possible, but some readers may
    /// always return an owned string.
    ///
    /// Does not advance the reader.
    ///
    /// Returns an error if the data contains invalid characters.
    fn to_string(&self) -> Result<Cow<str>>;

    /// Convert all remaining data to a clone-on-write string, including invalid characters.
    ///
    /// The string will be borrowed where possible, but some readers may
    /// always return an owned string.
    ///
    /// Does not advance the reader.
    fn to_string_lossy(&self) -> Result<Cow<str>>;

    /// Read a u8 array.
    fn read_u8_array<A>(&mut self) -> Result<A>
    where
        A: Sized + Default + AsMut<[u8]>;

    /// Read a u8.
    fn read_u8(&mut self) -> Result<u8>;

    /// Read an i8.
    fn read_i8(&mut self) -> Result<i8>;

    /// Read a u16.
    fn read_u16(&mut self) -> Result<u16>;

    /// Read an i16.
    fn read_i16(&mut self) -> Result<i16>;

    /// Read a u32.
    fn read_u32(&mut self) -> Result<u32>;

    /// Read an i32.
    fn read_i32(&mut self) -> Result<i32>;

    /// Read a u64.
    fn read_u64(&mut self) -> Result<u64>;

    /// Read an i64.
    fn read_i64(&mut self) -> Result<i64>;

    /// Read a null-terminated slice, and return it (excluding the null).
    fn read_null_terminated_slice(&mut self) -> Result<Self> {
        let idx = self.find(0)?;
        let val = self.split(idx)?;
        self.skip(Self::Offset::from_u8(1))?;
        Ok(val)
    }

    /// Read an unsigned LEB128 encoded integer.
    fn read_uleb128(&mut self) -> Result<u64> {
        leb128::read::unsigned(self)
    }

    /// Read a signed LEB128 encoded integer.
    fn read_sleb128(&mut self) -> Result<i64> {
        leb128::read::signed(self)
    }

    /// Read an address-sized integer, and return it as a `u64`.
    fn read_address(&mut self, address_size: u8) -> Result<u64> {
        match address_size {
            1 => self.read_u8().map(|v| v as u64),
            2 => self.read_u16().map(|v| v as u64),
            4 => self.read_u32().map(|v| v as u64),
            8 => self.read_u64(),
            otherwise => Err(Error::UnsupportedAddressSize(otherwise)),
        }
    }

    /// Parse a word-sized integer according to the DWARF format, and return it as a `u64`.
    fn read_word(&mut self, format: Format) -> Result<u64> {
        match format {
            Format::Dwarf32 => self.read_u32().map(|v| v as u64),
            Format::Dwarf64 => self.read_u64(),
        }
    }

    /// Parse a word-sized integer according to the DWARF format, and return it as an offset.
    fn read_offset(&mut self, format: Format) -> Result<Self::Offset> {
        self.read_word(format).and_then(Self::Offset::from_u64)
    }
}
