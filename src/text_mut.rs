use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
    str::Utf8Error,
};

use bytes::BytesMut;

use crate::Text;

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

    /// Copies the provided string into a new mutable buffer.
    pub fn copy_from(s: impl AsRef<str>) -> Self {
        // There is no `BytesMut::copy_from_slice`
        let s = s.as_ref();
        let mut t = Self::with_capacity(s.len());
        t.push_str(s);
        t
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
        // Safety: self.0 is guaranteed to be valid UTF-8
        unsafe { Text::from_utf8_unchecked(self.0.freeze()) }
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
        // Safety:
        // `self` will always contain valid UTF-8
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

// ### Self comparisons

impl PartialEq for TextMut {
    fn eq(&self, other: &Self) -> bool {
        (&**self).eq(&**other)
    }
}

impl Eq for TextMut {}

impl PartialEq<&TextMut> for TextMut {
    fn eq(&self, other: &&TextMut) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialEq<&mut TextMut> for TextMut {
    fn eq(&self, other: &&mut TextMut) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialOrd for TextMut {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ### str comparisons

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

impl PartialEq<&mut str> for TextMut {
    fn eq(&self, other: &&mut str) -> bool {
        (&**self).eq(*other)
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

impl PartialOrd<&mut str> for TextMut {
    fn partial_cmp(&self, other: &&mut str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(*other)
    }
}

// ### String comparisons

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

impl PartialEq<&mut String> for TextMut {
    fn eq(&self, other: &&mut String) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialOrd<String> for TextMut {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

impl PartialOrd<&String> for TextMut {
    fn partial_cmp(&self, other: &&String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

impl PartialOrd<&mut String> for TextMut {
    fn partial_cmp(&self, other: &&mut String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

// ### Text comparisons

impl PartialEq<Text> for TextMut {
    fn eq(&self, other: &Text) -> bool {
        (&**self).eq(&**other)
    }
}

impl PartialEq<&Text> for TextMut {
    fn eq(&self, other: &&Text) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialEq<&mut Text> for TextMut {
    fn eq(&self, other: &&mut Text) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialOrd<Text> for TextMut {
    fn partial_cmp(&self, other: &Text) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

impl PartialOrd<&Text> for TextMut {
    fn partial_cmp(&self, other: &&Text) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

impl PartialOrd<&mut Text> for TextMut {
    fn partial_cmp(&self, other: &&mut Text) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

/// ## Hash

impl Hash for TextMut {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state);
    }
}

/// ## Extend

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_mut_create() {
        let mut buf = TextMut::new();
        buf.push_str("Hello, ");
        buf.push_str("world!");
        assert_eq!(buf, "Hello, world!");
        buf.push(' ');
        buf.extend(vec!["Woo, ", "unit tests!"]);
        assert_eq!(buf, "Hello, world! Woo, unit tests!");
        buf.extend(vec![String::from(" I'm in a String.")]);
        buf.extend(vec![Text::from(" Ooo, text")]);
        assert_eq!(
            format!("{}", buf),
            String::from("Hello, world! Woo, unit tests! I'm in a String. Ooo, text")
        );
        buf.clear();
        assert_eq!(buf, "");
    }
}
