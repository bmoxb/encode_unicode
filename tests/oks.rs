/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Test that every method gives the correct result for valid values.
//! Except iterators, which are stateful.

use std::char;
use std::str::{self,FromStr};
use std::cmp::Ordering;
use std::hash::{Hash,Hasher};
use std::collections::hash_map::DefaultHasher;
#[allow(deprecated,unused)]
use std::ascii::AsciiExt;
use std::iter::FromIterator;
extern crate encode_unicode;
use encode_unicode::*;


#[test]
fn equal_defaults() {
    assert_eq!(Utf8Char::default().to_char(), char::default());
    assert_eq!(Utf16Char::default().to_char(), char::default());
}

#[test]
fn same_size_as_char() {
    use std::mem::size_of;
    assert_eq!(size_of::<Utf8Char>(), size_of::<char>());
    assert_eq!(size_of::<Utf16Char>(), size_of::<char>());
}

#[test]
#[cfg(feature="std")]
fn read_iterator() {
    use std::io::Read;
    use std::cmp::min;

    let uc = 'ä'.to_utf8();
    assert_eq!(uc.len(), 2);
    for chunk in 1..5 {
        let mut buf = [b'E'; 6];
        let mut iter = uc.into_iter();
        let mut written = 0;
        for _ in 0..4 {
            assert_eq!(iter.read(&mut buf[..0]).unwrap(), 0);
            let wrote = iter.read(&mut buf[written..written+chunk]).unwrap();
            assert_eq!(wrote, min(2-written, chunk));
            written += wrote;
            for &b in &buf[written..] {assert_eq!(b, b'E');}
            assert_eq!(buf[..written], AsRef::<[u8]>::as_ref(&uc)[..written]);
        }
        assert_eq!(written, 2);
    }
}


const EDGES_AND_BETWEEN: [char;19] = [
    '\u{0}',// min
    '\u{3b}',// middle ASCII
    'A',// min ASCII uppercase
    'N',// middle ASCII uppercase
    'Z',// max ASCII uppercase
    'a',// min ASCII lowercase
    'm',// middle ASCII lowercase
    'z',// max ASCII lowercase
    '\u{7f}',// max ASCII and 1-byte UTF-8
    '\u{80}',// min 2-byte UTF-8
    '\u{111}',// middle
    '\u{7ff}',// max 2-byte UTF-8
    '\u{800}',// min 3-byte UTF-8
    '\u{d7ff}',// before reserved
    '\u{e000}',// after reserved
    '\u{ffff}',// max UTF-16 single and 3-byte UTF-8
    '\u{10000}',// min UTF-16 surrogate and 4-byte UTF-8
    '\u{abcde}',// middle
    '\u{10ffff}',// max
];

fn eq_cmp_hash(c: char) -> (Utf8Char, Utf16Char) {
    fn hash<T:Hash>(v: T) -> u64 {
        #[allow(deprecated)]
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }
    let u8c = c.to_utf8();
    assert_eq!(u8c.to_char(), c);
    assert_eq!(u8c, u8c);
    assert_eq!(hash(u8c), hash(u8c));
    assert_eq!(u8c.cmp(&u8c), Ordering::Equal);
    assert!(u8c.eq_ignore_ascii_case(&u8c));
    let u16c = c.to_utf16();
    assert_eq!(u16c.to_char(), c);
    assert_eq!(u16c, u16c);
    assert_eq!(hash(u16c), hash(c));
    assert_eq!(u16c.cmp(&u16c), Ordering::Equal);
    assert!(u16c.eq_ignore_ascii_case(&u16c));

    for &other in &EDGES_AND_BETWEEN {
        let u8other = other.to_utf8();
        assert_eq!(u8c == u8other,  c == other);
        assert_eq!(hash(u8c)==hash(u8other),  hash(c)==hash(other));
        assert_eq!(u8c.cmp(&u8other), c.cmp(&other));
        assert_eq!(u8c.eq_ignore_ascii_case(&u8other), c.eq_ignore_ascii_case(&other));

        let u16other = other.to_utf16();
        assert_eq!(u16c == u16other,  c == other);
        assert_eq!(hash(u16c)==hash(u16other),  hash(c)==hash(other));
        assert_eq!(u16c.cmp(&u16other), c.cmp(&other));
        assert_eq!(u16c.eq_ignore_ascii_case(&u16other), c.eq_ignore_ascii_case(&other));
    }
    (u8c, u16c)
}

fn iterators(c: char) {
    let mut iter = c.iter_utf8_bytes();
    let mut buf = [0; 4];
    let mut iter_ref = c.encode_utf8(&mut buf[..]).as_bytes().iter();
    for _ in 0..6 {
        assert_eq!(iter.size_hint(), iter_ref.size_hint());
        assert_eq!(format!("{:?}", iter), format!("{:?}", iter_ref.as_slice()));
        assert_eq!(iter.next(), iter_ref.next().cloned());
    }

    let mut iter = c.iter_utf16_units();
    let mut buf = [0; 2];
    let mut iter_ref = c.encode_utf16(&mut buf[..]).iter();
    for _ in 0..4 {
        assert_eq!(iter.size_hint(), iter_ref.size_hint());
        assert_eq!(format!("{:?}", iter), format!("{:?}", iter_ref.as_slice()));
        assert_eq!(iter.next(), iter_ref.next().cloned());
    }
}

fn test(c: char) {
    assert_eq!(char::from_u32(c as u32), Some(c));
    assert_eq!(char::from_u32_detailed(c as u32), Ok(c));
    assert_eq!(unsafe{ char::from_u32_unchecked(c as u32) }, c);
    let (u8c, u16c) = eq_cmp_hash(c);
    iterators(c);
    assert_eq!(Utf16Char::from(u8c), u16c);
    assert_eq!(Utf8Char::from(u16c), u8c);
    let utf8_len = c.len_utf8();
    let utf16_len = c.len_utf16();
    let mut as_str = c.to_string();

    // UTF-8
    let mut buf = [0; 4];
    let reference = c.encode_utf8(&mut buf[..]).as_bytes();
    let len = reference.len(); // short name because it is used in many places.
    assert_eq!(len, utf8_len);
    assert_eq!(reference[0].extra_utf8_bytes(), Ok(len-1));
    assert_eq!(reference[0].extra_utf8_bytes_unchecked(), len-1);
    assert_eq!(AsRef::<[u8]>::as_ref(&u8c), reference);

    let (arr,arrlen) = u8c.to_array();
    assert_eq!(arrlen, len);
    assert_eq!(Utf8Char::from_array(arr), Ok(u8c));
    assert_eq!(c.to_utf8_array(),  (arr, len));

    let str_ = str::from_utf8(reference).unwrap();
    let ustr = Utf8Char::from_str(str_).unwrap();
    assert_eq!(ustr.to_array().0, arr);// bitwise equality
    assert_eq!(char::from_utf8_array(arr), Ok(c));
    let mut longer = [0xff; 5]; // 0xff is never valid
    longer[..len].copy_from_slice(reference);
    assert_eq!(char::from_utf8_slice_start(reference), Ok((c,len)));
    assert_eq!(char::from_utf8_slice_start(&longer), Ok((c,len)));
    assert_eq!(Utf8Char::from_slice_start(reference), Ok((u8c,len)));
    assert_eq!(Utf8Char::from_slice_start(&longer), Ok((u8c,len)));
    for other in &mut longer[len..] {*other = b'?'}
    assert_eq!(Utf8Char::from_str(str_), Ok(u8c));
    assert_eq!(Utf8Char::from_str_start(str_), Ok((u8c,len)));
    assert_eq!(Utf8Char::from_str_start(str::from_utf8(&longer).unwrap()), Ok((u8c,len)));
    unsafe {
        // Hopefully make bugs easier to catch by making reads into unallocated memory by filling
        // a jemalloc bin. See table on http://jemalloc.net/jemalloc.3.html for bin sizes.
        // I have no idea whether this works.
        let mut boxed = Box::new([0xffu8; 16]);
        let start = boxed.len()-len; // reach the end
        boxed[start..].copy_from_slice(reference);
        let slice = &boxed[start..start]; // length of slice should be ignored.
        assert_eq!(Utf8Char::from_slice_start_unchecked(slice), (u8c,len));
    }
    assert_eq!(&Vec::<u8>::from_iter(Some(u8c))[..], reference);
    assert_eq!(&String::from_iter(Some(u8c))[..], str_);
    assert_eq!(format!("{:?}", u8c), format!("{:?}", c));
    assert_eq!(format!("{}", u8c), format!("{}", c));
    assert_eq!(u8c.is_ascii(), c.is_ascii());
    assert_eq!(u8c.to_ascii_lowercase().to_char(), c.to_ascii_lowercase());
    assert_eq!(u8c.to_ascii_uppercase().to_char(), c.to_ascii_uppercase());

    // UTF-16
    let mut buf = [0; 2];
    let reference = c.encode_utf16(&mut buf[..]);
    let len = reference.len();
    assert_eq!(len, utf16_len);
    assert_eq!(reference[0].utf16_needs_extra_unit(), Ok(len==2));
    assert_eq!(reference[0].is_utf16_leading_surrogate(), len==2);
    assert_eq!(u16c.as_ref(), reference);
    let mut longer = [0; 3];
    longer[..len].copy_from_slice(reference);
    assert_eq!(char::from_utf16_slice_start(reference), Ok((c,len)));
    assert_eq!(char::from_utf16_slice_start(&longer), Ok((c,len)));
    assert_eq!(Utf16Char::from_slice_start(reference), Ok((u16c,len)));
    assert_eq!(Utf16Char::from_slice_start(&longer), Ok((u16c,len)));
    assert_eq!(Utf16Char::from_str(&as_str), Ok(u16c));
    as_str.push(c);
    assert_eq!(Utf16Char::from_str_start(&as_str), Ok((u16c,utf8_len)));
    unsafe {
        // Hopefully make bugs easier to catch by making reads into unallocated memory by filling
        // a jemalloc bin. See table on http://jemalloc.net/jemalloc.3.html for bin sizes.
        // I have no idea whether this works.
        let mut boxed = Box::new([0u16; 8]);
        let start = boxed.len()-len; // reach the end
        boxed[start..].copy_from_slice(reference);
        let slice = &boxed[start..start]; // length of slice should be ignored.
        assert_eq!(Utf16Char::from_slice_start_unchecked(slice), (u16c,len));
    }
    let tuple = c.to_utf16_tuple();
    assert_eq!(tuple, (reference[0],reference.get(1).cloned()));
    assert_eq!(char::from_utf16_tuple(tuple), Ok(c));
    assert_eq!(c.to_utf16().to_char(), c);
    assert_eq!(&Vec::<u16>::from_iter(Some(u16c))[..], reference);
    assert_eq!(format!("{:?}", u16c), format!("{:?}", c));
    assert_eq!(format!("{}", u16c), format!("{}", c));
    assert_eq!(u16c.is_ascii(), c.is_ascii());
    assert_eq!(u16c.to_ascii_lowercase().to_char(), c.to_ascii_lowercase());
    assert_eq!(u16c.to_ascii_uppercase().to_char(), c.to_ascii_uppercase());
}


#[test]
fn edges_middle() {
    for &c in &EDGES_AND_BETWEEN {
        test(c);
    }
}


#[test]
#[ignore]
fn all() {
    for cp in std::iter::Iterator::chain(0..0xd800, 0xe000..0x110000) {
        let c = char::from_u32(cp).expect("not a valid char");
        test(c);
    }
}
