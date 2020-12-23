// Copied from https://github.com/Alexhuszagh/rust-nom-unicode/commit/cb72d173d7dd8b580760541137458a481b365b33
// Nom-Unicode is dual licensed under the Apache 2.0 license as well as the MIT license.
// https://github.com/Alexhuszagh/rust-nom-unicode/blob/master/LICENSE-MIT
// https://github.com/Alexhuszagh/rust-nom-unicode/blob/master/LICENSE-APACHE

//! Adaptors to add Unicode-aware parsing to Nom.

use nom::AsChar;

// HELPERS

/// nom::AsChar for only unicode-aware character types.
pub trait IsChar: AsChar {}

impl IsChar for char {}

impl<'a> IsChar for &'a char {}

// Generates `is_x` implied helper functions.
macro_rules! is_impl {
    ($($name:ident)*) => ($(
        #[inline(always)]
        fn $name<T: IsChar>(item: T) -> bool {
            item.as_char().$name()
        }
    )*);
}

is_impl! {
    is_alphabetic
    is_lowercase
    is_uppercase
    is_whitespace
    is_alphanumeric
    is_control
    is_numeric
    is_ascii
}

// Macro to dynamically document a generated function.
macro_rules! doc {
    ($x:expr, $item:item) => (
        #[doc = $x]
        $item
    );
}

// COMPLETE

/// Nom complete parsing API functions.
#[allow(dead_code)]
pub mod complete {
  use super::*;
  use nom::error::{ErrorKind, ParseError};
  use nom::{IResult, InputTakeAtPosition};

  // Dynamically generate both the zero and 1 parse APIs.
  macro_rules! parse_impl {
        ($($name0:ident, $name1:ident, $kind:ident, $callback:ident, $comment:expr)*) => ($(
            doc!(concat!("Recognizes zero or more ", $comment),
                #[inline]
                pub fn $name0<T, Error>(input: T)
                    -> IResult<T, T, Error>
                    where T: InputTakeAtPosition,
                          <T as InputTakeAtPosition>::Item: IsChar,
                          Error: ParseError<T>
                {
                  input.split_at_position_complete(|item| !$callback(item))
                }
            );

            doc!(concat!("Recognizes one or more ", $comment),
                #[inline]
                pub fn $name1<T, Error>(input: T)
                    -> IResult<T, T, Error>
                    where T: InputTakeAtPosition,
                          <T as InputTakeAtPosition>::Item: IsChar,
                          Error: ParseError<T>
                {
                  input.split_at_position1_complete(|item| !$callback(item), ErrorKind::$kind)
                }
            );
        )*);
    }

  parse_impl! {
      alpha0,         alpha1,         Alpha,          is_alphabetic,      "lowercase and uppercase alphabetic Unicode characters."
      lower0,         lower1,         Alpha,          is_lowercase,       "lowercase alphabetic Unicode characters."
      upper0,         upper1,         Alpha,          is_uppercase,       "lowercase alphabetic Unicode characters."
      space0,         space1,         Space,          is_whitespace,      "whitespace Unicode characters."
      alphanumeric0,  alphanumeric1,  AlphaNumeric,   is_alphanumeric,    "alphabetic and numeric Unicode characters."
      control0,       control1,       TakeWhile1,     is_control,         "control Unicode characters."
      digit0,         digit1,         Digit,          is_numeric,         "numeric Unicode characters."
      ascii0,         ascii1,         TakeWhile1,     is_ascii,           "ASCII characters."
  }
}
