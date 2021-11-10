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
    ops::{Deref, DerefMut, RangeBounds},
    str::Utf8Error,
};

use bytes::{Bytes, BytesMut};

// # Immutable

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

// ## Conversions

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

// ## Formatting

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

// ## Comparisons

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

impl PartialEq<TextMut> for Text {
    fn eq(&self, other: &TextMut) -> bool {
        (&**self).eq(&**other)
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

impl PartialOrd<TextMut> for Text {
    fn partial_cmp(&self, other: &TextMut) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
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

// # Immutable

/// Mutable UTF-8 text buffer
#[derive(Default)]
pub struct TextMut(BytesMut);

impl TextMut {
    /// Creates a new, empty, text buffer.
    pub fn new() -> Self {
        Self(BytesMut::new())
    }

    /// Creates a new, empty, text buffer that can grow to at least `capacity`
    /// bytes long before reallocating
    pub fn with_capacity(capacity: usize) -> Self {
        Self(BytesMut::with_capacity(capacity))
    }

    /// Converts `Bytes` to `Text`.
    pub fn from_utf8(b: BytesMut) -> Result<Self, Utf8Error> {
        // run utf-8 validation
        let _ = std::str::from_utf8(b.as_ref())?;
        Ok(Self(b))
    }

    /// Converts `Bytes` to `Text` without verifying that it's valid UTF-8
    /// # Safety
    /// Must be valid UTF-8
    #[inline]
    pub const unsafe fn from_utf8_unchecked(b: BytesMut) -> Self {
        Self(b)
    }

    /// The number of bytes in this text
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The maximum length of this buffer before reallocation is required
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Checks if this text is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Freezes this into an immutable, shareable, text buffer.
    pub fn freeze(self) -> Text {
        Text(self.0.freeze())
    }

    /// Reserves space for at least `additional` more bytes to be inserted
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Clears the buffer of its contents
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Get a reference to the inner bytes
    pub fn as_bytes(&self) -> &BytesMut {
        &self.0
    }

    /// Get a mutable reference to the inner bytes
    /// # Safety
    /// The returned reference must not be modified in such a way that makes it
    /// invalid UTF-8, even momentarily
    pub unsafe fn as_bytes_mut(&mut self) -> &mut BytesMut {
        &mut self.0
    }

    /// Convert into a mutable buffer of raw bytes
    pub fn into_bytes_mut(self) -> BytesMut {
        self.0
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

    /// Copies the string reference into this buffer
    pub fn push_str(&mut self, s: impl AsRef<str>) {
        self.0.extend_from_slice(s.as_ref().as_bytes())
    }

    /// Adds a character to the end of this buffer
    pub fn push(&mut self, c: char) {
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        self.push_str(s);
    }

    /// Joins two `TextMut`s together
    ///
    /// If they were once contiguous (i.e. from one of the `split` methods) then
    /// this takes O(1) time
    pub fn join(&mut self, other: TextMut) {
        self.0.unsplit(other.0)
    }

    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
    }

    fn as_str_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.0.as_mut()) }
    }
}

// ## Conversions

impl AsRef<str> for TextMut {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl AsMut<str> for TextMut {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

impl Deref for TextMut {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for TextMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl TryFrom<BytesMut> for TextMut {
    type Error = Utf8Error;

    fn try_from(b: BytesMut) -> Result<Self, Self::Error> {
        Self::from_utf8(b)
    }
}

impl From<TextMut> for BytesMut {
    fn from(t: TextMut) -> Self {
        t.into_bytes_mut()
    }
}

impl From<TextMut> for Text {
    fn from(t: TextMut) -> Self {
        t.freeze()
    }
}

// ## Formatting

impl Display for TextMut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for TextMut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

// ## Comparisons

impl PartialEq for TextMut {
    fn eq(&self, other: &Self) -> bool {
        (&**self).eq(&**other)
    }
}

impl Eq for TextMut {}

impl PartialEq<str> for TextMut {
    fn eq(&self, other: &str) -> bool {
        (&**self).eq(other)
    }
}
impl PartialEq<&str> for TextMut {
    fn eq(&self, other: &&str) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialEq<String> for TextMut {
    fn eq(&self, other: &String) -> bool {
        (&**self).eq(other)
    }
}

impl PartialEq<&String> for TextMut {
    fn eq(&self, other: &&String) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialEq<Text> for TextMut {
    fn eq(&self, other: &Text) -> bool {
        (&**self).eq(&**other)
    }
}

impl PartialOrd for TextMut {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<str> for TextMut {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(other)
    }
}

impl PartialOrd<&str> for TextMut {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(*other)
    }
}

impl PartialOrd<Text> for TextMut {
    fn partial_cmp(&self, other: &Text) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

impl Ord for TextMut {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&**self).cmp(&**other)
    }
}

impl Hash for TextMut {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state);
    }
}

impl Extend<char> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = char>,
    {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(move |c| self.push(c));
    }
}

impl<'a> Extend<&'a str> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a str>,
    {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl Extend<String> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = String>,
    {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl<'a> Extend<&'a String> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a String>,
    {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl Extend<Text> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Text>,
    {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl<'a> Extend<&'a Text> for TextMut {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a Text>,
    {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}
