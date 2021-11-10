//! `Text` is to [`Bytes`](https://docs.rs/bytes/1/bytes/struct.Bytes.html) what `String` is to `Vec<u8>`
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

mod text;
mod text_mut;

pub use text::Text;
pub use text_mut::TextMut;
