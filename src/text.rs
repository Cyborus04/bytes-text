use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, RangeBounds},
    str::Utf8Error,
};

use bytes::Bytes;

use crate::TextMut;

/// Immutable, reference counted, UTF-8 text
///
/// # Example
///
/// ```
/// # use bytes_text::Text;
/// # use bytes::{Bytes, Buf};
/// let mut s = String::from("Text, woo!");
///
/// let text = Text::from(s);
/// assert_eq!(text, "Text, woo!");
/// let (a, b) = text.split_at(5).unwrap();
/// assert_eq!(a, "Text,");
/// assert_eq!(b, " woo!");
/// ```
#[derive(Default, Clone)]
pub struct Text(Bytes);

impl Text {
    /// Creates a new, empty, text buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let mut text = Text::new();
    /// assert!(text.is_empty());
    /// ```
    pub fn new() -> Self {
        Self(Bytes::new())
    }

    /// Converts `Bytes` to `Text`.
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// # use bytes::{Bytes, Buf};
    /// let mut buf = Bytes::from_static(&[
    ///     0x69, 0x27, 0x6d, 0x20, 0x69, 0x6e, 0x20, 0x61,
    ///     0x20, 0x62, 0x75, 0x66, 0x66, 0x65, 0x72, 0x21
    /// ]);
    ///
    /// let text = Text::from_utf8(buf).unwrap();
    /// assert_eq!(text, "i'm in a buffer!");
    /// ```
    pub fn from_utf8(b: Bytes) -> Result<Self, Utf8Error> {
        // run utf-8 validation
        let _ = std::str::from_utf8(b.as_ref())?;
        Ok(Self(b))
    }

    /// Converts `Bytes` to `Text` without verifying that it's valid UTF-8
    ///
    /// # Safety
    ///
    /// Must be valid UTF-8
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// # use bytes::{Bytes, Buf};
    /// let mut buf = Bytes::from_static(&[
    ///     0x69, 0x27, 0x6d, 0x20, 0x69, 0x6e, 0x20, 0x61,
    ///     0x20, 0x62, 0x75, 0x66, 0x66, 0x65, 0x72, 0x21
    /// ]);
    ///
    /// let text = unsafe { Text::from_utf8_unchecked(buf) };
    /// assert_eq!(text, "i'm in a buffer!");
    /// ```
    #[inline]
    pub const unsafe fn from_utf8_unchecked(b: Bytes) -> Self {
        Self(b)
    }

    /// Copies the provided string into a new buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let mut s = format!("the answer is: {}", 42);
    /// let text = Text::copy_from(s);
    /// assert_eq!(text, "the answer is: 42");
    /// ```
    pub fn copy_from(s: impl AsRef<str>) -> Self {
        // copy the bytes and wrap it
        // guaranteed to be valid
        Self(Bytes::copy_from_slice(s.as_ref().as_bytes()))
    }

    /// Creates `Text` from a static `str`
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let text = Text::from_static("Static!");
    /// ```
    /// Can also use `from`
    /// ```
    /// # use bytes_text::Text;
    /// let text = Text::from("Also static!");
    /// ```
    pub const fn from_static(s: &'static str) -> Self {
        Self(Bytes::from_static(s.as_bytes()))
    }

    /// The number of bytes in this text
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let text = Text::from_static("Hello!");
    /// assert_eq!(text.len(), 6);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if this text is empty
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let text = Text::new();
    /// assert!(text.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the inner bytes
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// # use bytes::Bytes;
    /// let text = Text::from("Woah");
    /// let bytes: &Bytes = text.as_bytes();
    /// assert_eq!(bytes, &b"Woah"[..])
    /// ```
    pub fn as_bytes(&self) -> &Bytes {
        &self.0
    }

    /// Convert into bytes
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// # use bytes::Bytes;
    /// let text = Text::from("Woah");
    /// let bytes: Bytes = text.into_bytes();
    /// assert_eq!(&bytes, &b"Woah"[..])
    /// ```
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Get a sub-body of text
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// # use bytes::Bytes;
    /// let text = Text::from("Hi, I'm some text!");
    /// let end = text.get(4..).unwrap();
    /// assert_eq!(end, "I'm some text!");
    /// let middle = text.get(8..12).unwrap();
    /// assert_eq!(middle, "some");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let text = Text::from("Woo, split!");
    /// let (a, b) = text.split_at(4).unwrap();
    /// assert_eq!(a, "Woo,");
    /// assert_eq!(b, " split!");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let mut text = Text::from("Woo, split!");
    /// let end = text.split_off(4).unwrap();
    /// assert_eq!(text, "Woo,");
    /// assert_eq!(end, " split!");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use bytes_text::Text;
    /// let mut text = Text::from("Woo, split!");
    /// let start = text.split_to(4).unwrap();
    /// assert_eq!(start, "Woo,");
    /// assert_eq!(text, " split!");
    /// ```
    pub fn split_to(&mut self, index: usize) -> Option<Self> {
        soft_assert::soft_assert!(self.is_char_boundary(index));
        let right = self.0.split_to(index);
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

// ### Self comparisons

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        (&**self).eq(&**other)
    }
}

impl Eq for Text {}

impl PartialEq<&Text> for Text {
    fn eq(&self, other: &&Text) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialEq<&mut Text> for Text {
    fn eq(&self, other: &&mut Text) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Text {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&**self).cmp(&**other)
    }
}
// ### str comparisons

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

impl PartialEq<&mut str> for Text {
    fn eq(&self, other: &&mut str) -> bool {
        (&**self).eq(*other)
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

impl PartialOrd<&mut str> for Text {
    fn partial_cmp(&self, other: &&mut str) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(*other)
    }
}

// ### String comparisons

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

impl PartialEq<&mut String> for Text {
    fn eq(&self, other: &&mut String) -> bool {
        (&**self).eq(*other)
    }
}

impl PartialOrd<String> for Text {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

impl PartialOrd<&String> for Text {
    fn partial_cmp(&self, other: &&String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

impl PartialOrd<&mut String> for Text {
    fn partial_cmp(&self, other: &&mut String) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

// ### TextMut Comparisons

impl PartialEq<TextMut> for Text {
    fn eq(&self, other: &TextMut) -> bool {
        (&**self).eq(&**other)
    }
}

impl PartialEq<&TextMut> for Text {
    fn eq(&self, other: &&TextMut) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialEq<&mut TextMut> for Text {
    fn eq(&self, other: &&mut TextMut) -> bool {
        (&**self).eq(&***other)
    }
}

impl PartialOrd<TextMut> for Text {
    fn partial_cmp(&self, other: &TextMut) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&**other)
    }
}

impl PartialOrd<&TextMut> for Text {
    fn partial_cmp(&self, other: &&TextMut) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

impl PartialOrd<&mut TextMut> for Text {
    fn partial_cmp(&self, other: &&mut TextMut) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(&***other)
    }
}

// ## Hash

impl Hash for Text {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state);
    }
}
