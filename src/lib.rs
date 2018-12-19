//! This crate implements a `Read` adapter that converts the invalid JSON
//! tokens `NaN` and `Infinity` into other tokens without otherwise distorting
//! the stream.  It achieves this by converting `NaN` and `Infinity` into `0.0`.
//! 
//! This is useful because the Python JSON library traditionally emits invalid
//! JSON if `NaN` and `Infinity` values are encountered.  If you have to support
//! clients like this, this wrapper can be used to still deserialize such a
//! JSON document.
//! 
//! This is just a way to get this to parse and `0.0` is the only value that can
//! be inserted in a standardized way that fits without changing any of the
//! positions.
//! 
//! # Example Conversion
//! 
//! The following JSON document:
//! 
//! ```ignore
//! {"nan":NaN,"inf":Infinity,"-inf":-Infinity}
//! ```
//! 
//! is thus converted to:
//! 
//! ```ignore
//! {"nan":0.0,"inf":0.0     ,"-inf":-0.0     }
//! ```
//! 
//! # serde support
//! 
//! If the `serde` feature is enabled then the crate provides some basic
//! wrappers around `serde_json` to deserialize quickly and also by running
//! the conversions.
use std::io::{self, Read};
use std::fmt;

#[cfg(feature = "serde")]
mod serde_impl;
#[cfg(feature = "serde")]
pub use self::serde_impl::*;

#[derive(Copy, Clone)]
enum State {
    Initial,
    Quoted,
    QuotedEscape,
    NaN0,
    NaN1,
    Infinity0,
    Infinity1,
    Infinity2,
    Infinity3,
    Infinity4,
    Infinity5,
    Infinity6,
}

/// A reader that transparently translates python JSON compat tokens.
pub struct JsonCompatRead<R> {
    reader: R,
    state: State,
}

impl<R: Read> fmt::Debug for JsonCompatRead<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonCompatRead").finish()
    }
}

impl<R: Read> JsonCompatRead<R> {
    /// Wraps another reader.
    pub fn wrap(reader: R) -> JsonCompatRead<R> {
        JsonCompatRead { reader, state: State::Initial }
    }
}

impl<R: Read> Read for JsonCompatRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = io::Read::read(&mut self.reader, buf)?;
        self.state = translate_slice_impl(&mut buf[..read], self.state);
        Ok(read)
    }
}

fn translate_slice_impl(bytes: &mut [u8], mut state: State) -> State {
    for c in bytes {
        let rv = match (state, *c) {
            (State::Initial, b'N') => (State::NaN0, b'0'),
            (State::NaN0, b'a') => (State::NaN1, b'.'),
            (State::NaN1, b'N') => (State::Initial, b'0'),
            (State::Initial, b'I') => (State::Infinity0, b'0'),
            (State::Infinity0, b'n') => (State::Infinity1, b'.'),
            (State::Infinity1, b'f') => (State::Infinity2, b'0'),
            (State::Infinity2, b'i') => (State::Infinity3, b' '),
            (State::Infinity3, b'n') => (State::Infinity4, b' '),
            (State::Infinity4, b'i') => (State::Infinity5, b' '),
            (State::Infinity5, b't') => (State::Infinity6, b' '),
            (State::Infinity6, b'y') => (State::Initial, b' '),
            (State::Initial, b'"') => (State::Quoted, b'"'),
            (State::Quoted, b'\\') => (State::QuotedEscape, b'\\'),
            (State::QuotedEscape, c) => (State::Quoted, c),
            (State::Quoted, b'"') => (State::Initial, b'"'),
            (state, c) => (state, c),
        };
        state = rv.0;
        *c = rv.1;
    }
    state
}

/// Translates a slice in place.
/// 
/// This works the same as the `JsonCompatRead` struct but instead converts a
/// slice in place.  This is useful when working with JSON in slices.
pub fn translate_slice(bytes: &mut [u8]) {
    translate_slice_impl(bytes, State::Initial);
}

#[test]
fn test_reader_simple() {
    let json = r#"{"nan":0.0,"inf":Infinity,"-inf":-Infinity}"#;
    assert_eq!(json.len(), 43);
    let mut rdr = JsonCompatRead::wrap(json.as_bytes());
    let mut rv = String::new();
    let read = rdr.read_to_string(&mut rv).unwrap();
    assert_eq!(read, 43);
    assert_eq!(rv, "{\"nan\":0.0,\"inf\":0.0     ,\"-inf\":-0.0     }");
}

#[test]
fn test_reader_string() {
    let json = r#"{"nan":"nan","Infinity":"-Infinity","other":NaN}"#;
    assert_eq!(json.len(), 48);
    let mut rdr = JsonCompatRead::wrap(json.as_bytes());
    let mut rv = String::new();
    let read = rdr.read_to_string(&mut rv).unwrap();
    assert_eq!(read, 48);
    assert_eq!(rv, "{\"nan\":\"nan\",\"Infinity\":\"-Infinity\",\"other\":0.0}");
}

#[test]
fn test_reader_string_escaping() {
    let json = r#""NaN\"NaN\"NaN""#;
    assert_eq!(json.len(), 15);
    let mut rdr = JsonCompatRead::wrap(json.as_bytes());
    let mut rv = String::new();
    let read = rdr.read_to_string(&mut rv).unwrap();
    assert_eq!(read, 15);
    assert_eq!(rv, r#""NaN\"NaN\"NaN""#);
}

#[test]
fn test_translate_slice() {
    let mut json = br#"{"nan":"nan","Infinity":"-Infinity","other":NaN}"#.to_vec();
    translate_slice(&mut json[..]);
    assert_eq!(&json[..], &b"{\"nan\":\"nan\",\"Infinity\":\"-Infinity\",\"other\":0.0}"[..]);
}