//! `Text` is to `Bytes` what `String` is to `Vec<u8>`
//!
//! # Example
//!
//! ```
//! use bytes_text::Text;
//!
//! let text = Text::from("Hello, world!");
//! println!("{}", text);
//!
//! let hello = text.get(..5).unwrap();
//! assert_eq!(hello, "Hello");
//!
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, RangeBounds},
    str::Utf8Error,
};

use bytes::Bytes;

/// Immutable, reference counted, UTF-8 text
#[derive(Default)]
pub struct Text(Bytes);

impl Text {
    /// Converts `Bytes` to `Text`.
    pub fn from_utf8(b: Bytes) -> Result<Self, Utf8Error> {
        // run utf-8 validation
        let _ = std::str::from_utf8(b.as_ref())?;
        Ok(Self(b))
    }

    /// Converts `Bytes` to `Text` without verifying that it's valid UTF-8
    /// # Safety
    /// Must be valid UTF-8
    #[inline]
    pub const unsafe fn from_utf8_unchecked(b: Bytes) -> Self {
        Self(b)
    }

    /// Copies the provided string into
    pub fn copy_from(s: &str) -> Self {
        // copy the bytes and wrap it
        // guaranteed to be valid
        Self(Bytes::copy_from_slice(s.as_bytes()))
    }

    /// Creates `Text` from a static `str`
    pub const fn from_static(s: &'static str) -> Self {
        Self(Bytes::from_static(s.as_bytes()))
    }

    /// The number of bytes in this text
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if this text is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the inner bytes
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    /// Convert into bytes
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Get a sub-body of text
    pub fn get(&self, r: impl RangeBounds<usize>) -> Option<Text> {
        let start = match r.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(&i) => i.checked_add(1)?,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            std::ops::Bound::Included(&i) => i.checked_add(1)?,
            std::ops::Bound::Excluded(&i) => i,
            std::ops::Bound::Unbounded => self.len(),
        };
        // str::is_char_boundary returns false if the index is out of bounds,
        // so there's no need to check for it here
        soft_assert::soft_assert!(self.is_char_boundary(start) && self.is_char_boundary(end));
        Some(Self(self.0.slice(start..end)))
    }

    /// Splits the text into two halves
    ///
    /// Returns `Err(self)` if the index is not a valid char boundary
    pub fn split_at(mut self, index: usize) -> Result<(Self, Self), Self> {
        soft_assert::soft_assert!(self.is_char_boundary(index), Err(self));
        let right = self.0.split_off(index);
        Ok((Self(self.0), Self(right)))
    }

    /// Splits the text into two halves, `self` being the start half and
    /// returning the end half
    ///
    /// Returns `None` if the index is not a valid char boundary. If this
    /// returns `None`, `self` remains unchanged.
    pub fn split_off(&mut self, index: usize) -> Option<Self> {
        soft_assert::soft_assert!(self.is_char_boundary(index));
        let right = self.0.split_off(index);
        Some(Self(right))
    }

    /// Splits the text into two halves, `self` being the end half and
    /// returning the start half
    ///
    /// Returns `None` if the index is not a valid char boundary. If this
    /// returns `None`, `self` remains unchanged.
    pub fn split_from(&mut self, index: usize) -> Option<Self> {
        soft_assert::soft_assert!(self.is_char_boundary(index));
        let right = self.0.split_off(index);
        Some(Self(right))
    }

    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
    }
}

// Conversions

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&'static str> for Text {
    fn from(s: &'static str) -> Self {
        Self::from_static(s)
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self(Bytes::from(s.into_bytes()))
    }
}

impl TryFrom<Bytes> for Text {
    type Error = Utf8Error;

    fn try_from(b: Bytes) -> Result<Self, Self::Error> {
        Self::from_utf8(b)
    }
}

impl From<Text> for Bytes {
    fn from(t: Text) -> Self {
        t.into_bytes()
    }
}

// Formatting

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

// Comparisons

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        (&**self).eq(&**other)
    }
}

impl Eq for Text {}

impl PartialEq<str> for Text {
    fn eq(&self, other: &str) -> bool {
        (&**self).eq(other)
    }
}
impl PartialEq<&str> for Text {
    fn eq(&self, other: &&str) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialEq<String> for Text {
    fn eq(&self, other: &String) -> bool {
        (&**self).eq(other)
    }
}

impl PartialEq<&String> for Text {
    fn eq(&self, other: &&String) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<str> for Text {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(other)
    }
}

impl PartialOrd<&str> for Text {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(*other)
    }
}

impl Ord for Text {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&**self).cmp(&**other)
    }
}

impl Hash for Text {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state);
    }
}
