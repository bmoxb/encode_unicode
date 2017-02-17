/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use Utf16Iterator;
use CharExt;
use Utf8Char;
use errors::{InvalidUtf16Slice,InvalidUtf16Tuple};
extern crate core;
use self::core::{hash,fmt,cmp};
use self::core::borrow::Borrow;
use self::core::ops::Deref;
#[cfg(feature="std")]
use self::core::iter::FromIterator;
#[cfg(feature="std")]
use std::ascii::AsciiExt;
#[cfg(feature="ascii")]
use self::core::char;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::{AsciiChar,ToAsciiChar,ToAsciiCharError};


// I don't think there is any good default value for char, but char does.
#[derive(Default)]
// char doesn't do anything more advanced than u32 for Eq/Ord, so we shouldn't either.
// When it's a single unit, the second is zero, so Eq works.
// Ord however, breaks on surrogate pairs.
#[derive(PartialEq,Eq)]
#[derive(Clone,Copy)]


/// An unicode codepoint stored as UTF-16.
///
/// It can be borrowed as an `u16` slice, and has the same size as `char`.
pub struct Utf16Char {
    units: [u16; 2],
}


  /////////////////////
 //conversion traits//
/////////////////////

impl From<char> for Utf16Char {
    fn from(c: char) -> Self {
        let (first, second) = c.to_utf16_tuple();
        Utf16Char{ units: [first, second.unwrap_or(0)] }
    }
}
impl From<Utf8Char> for Utf16Char {
    fn from(utf8: Utf8Char) -> Utf16Char {
        let (b, utf8_len) = utf8.to_array();
        match utf8_len {
            1 => Utf16Char{ units: [b[0] as u16, 0] },
            4 => {// need surrogate
                let mut first = 0xd800 - (0x01_00_00u32 >> 10) as u16;
                first += (b[0] as u16 & 0x07) << 8;
                first += (b[1] as u16 & 0x3f) << 2;
                first += (b[2] as u16 & 0x30) >> 4;
                let mut second = 0xdc00;
                second |= (b[2] as u16 & 0x0f) << 6;
                second |=  b[3] as u16 & 0x3f;
                Utf16Char{ units: [first, second] }
            },
            _ => { // 2 or 3
                let mut unit = ((b[0] as u16 & 0x1f) << 6) | (b[1] as u16 & 0x3f);
                if utf8_len == 3 {
                    unit = (unit << 6) | (b[2] as u16 & 0x3f);
                }
                Utf16Char{ units: [unit, 0] }
            },
        }
    }
}
impl From<Utf16Char> for char {
    fn from(uc: Utf16Char) -> char {
        unsafe{ char::from_utf16_tuple_unchecked(uc.to_tuple()) }
    }
}
impl IntoIterator for Utf16Char {
    type Item=u16;
    type IntoIter=Utf16Iterator;
    /// Iterate over the units.
    fn into_iter(self) -> Utf16Iterator {
        Utf16Iterator::from(self)
    }
}
#[cfg(feature="std")]
impl Extend<Utf16Char> for Vec<u16> {
    fn extend<I:IntoIterator<Item=Utf16Char>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for u16c in iter {
            self.push(u16c.units[0]);
            if u16c.units[1] != 0 {
                self.push(u16c.units[1]);
            }
        }
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf16Char> for Vec<u16> {
    fn from_iter<I:IntoIterator<Item=Utf16Char>>(iter: I) -> Self {
        let mut vec = Vec::new();
        vec.extend(iter);
        return vec;
    }
}


  /////////////////
 //getter traits//
/////////////////
impl AsRef<[u16]> for Utf16Char {
    fn as_ref(&self) -> &[u16] {
        &self.units[..self.len()]
    }
}
impl Borrow<[u16]> for Utf16Char {
    fn borrow(&self) -> &[u16] {
        self.as_ref()
    }
}
impl Deref for Utf16Char {
    type Target = [u16];
    fn deref(&self) -> &[u16] {
        self.as_ref()
    }
}


  ////////////////
 //ascii traits//
////////////////
#[cfg(feature="std")]
impl AsciiExt for Utf16Char {
    type Owned = Self;
    fn is_ascii(&self) -> bool {
        self.units[0] < 128
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        self.to_ascii_lowercase() == other.to_ascii_lowercase()
    }
    fn to_ascii_uppercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'a' as u16);
        if n < 26 {Utf16Char{ units: [n+b'A' as u16, 0] }}
        else      {*self}
    }
    fn to_ascii_lowercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'A' as u16);
        if n < 26 {Utf16Char{ units: [n+b'a' as u16, 0] }}
        else      {*self}
    }
    fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl From<AsciiChar> for Utf16Char {
    fn from(ac: AsciiChar) -> Self {
        Utf16Char{ units: [ac.as_byte() as u16, 0] }
    }
}
#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl ToAsciiChar for Utf16Char {
    fn to_ascii_char(self) -> Result<AsciiChar, ToAsciiCharError> {
        unsafe{ AsciiChar::from(char::from_u32_unchecked(self.units[0] as u32)) }
    }
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        AsciiChar::from_unchecked(self.units[0] as u8)
    }
}


  /////////////////////////////////////////////////////////
 //Genaral traits that cannot be derived to emulate char//
/////////////////////////////////////////////////////////
impl hash::Hash for Utf16Char {
    fn hash<H : hash::Hasher>(&self,  state: &mut H) {
        self.to_char().hash(state);
    }
}
impl fmt::Debug for Utf16Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), fmtr)
    }
}
impl cmp::PartialOrd for Utf16Char {
    fn partial_cmp(&self,  rhs: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}
impl cmp::Ord for Utf16Char {
    fn cmp(&self,  rhs: &Self) -> cmp::Ordering {
        // Shift the first unit by 0xd if surrogate, and 0 otherwise.
        // This ensures surrogates are always greater than 0xffff, and
        // that the second unit only affect the result when the first are equal.
        // Multiplying by a constant factor isn't enough because that factor
        // would have to be greater than 1023 and smaller than 5.5.
        let lhs = (self.units[0] as u32, self.units[1] as u32);
        let rhs = (rhs.units[0] as u32, rhs.units[1] as u32);
        let lhs = (lhs.0 << (lhs.1 >> 12)) + lhs.1;
        let rhs = (rhs.0 << (rhs.1 >> 12)) + rhs.1;
        lhs.cmp(&rhs)
    }
}


  ///////////////////////////////////////////////////////
 //pub impls that should be together for nicer rustdoc//
///////////////////////////////////////////////////////
impl Utf16Char {
    /// Validate and store the first UTF-16 codepoint in the slice.
    /// Also return how many units were needed.
    pub fn from_slice(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice> {
        char::from_utf16_slice(src).map(|(_,len)| {
            let second = if len==2 {src[1]} else {0};
            (Utf16Char{ units: [src[0], second] }, len)
        })
    }
    /// Validate and store a UTF-16 pair as returned from `char.to_utf16_tuple()`.
    pub fn from_tuple(utf16: (u16,Option<u16>)) -> Result<Self,InvalidUtf16Tuple> {
        char::from_utf16_tuple(utf16).map(|_|
            Utf16Char{ units: [utf16.0, utf16.1.unwrap_or(0)] }
        )
    }

    /// Returns 1 or 2.
    /// There is no `.is_emty()` because it would always return false.
    pub fn len(self) -> usize {
        1 + (self.units[1] as usize >> 15)
    }
    /// Is this codepoint an ASCII character?
    #[cfg(not(feature="std"))]
    pub fn is_ascii(&self) -> bool {
        self.units[0] <= 127
    }

    /// Convert from UTF-16 to UTF-32
    pub fn to_char(self) -> char {
        self.into()
    }
    /// Write the internal representation to a slice,
    /// and then returns the number of `u16`s written.
    ///
    /// # Panics
    /// Will panic the buffer is too small;
    /// You can get the required length from `.len()`,
    /// but a buffer of length two is always large enough.
    pub fn to_slice(self,  dst: &mut[u16]) -> usize {
        match self.len() {
            l if l > dst.len() => panic!("The provided buffer is too small."),
            2 => {dst[1] = self.units[1];
                  dst[0] = self.units[0];},
            1 => {dst[0] = self.units[0];},
            _ => unreachable!()
        }
        self.len()
    }
    /// The second `u16` is used for surrogate pairs.
    pub fn to_tuple(self) -> (u16,Option<u16>) {
        (self.units[0],  if self.len()==2 {Some(self.units[1])} else {None})
    }
}
