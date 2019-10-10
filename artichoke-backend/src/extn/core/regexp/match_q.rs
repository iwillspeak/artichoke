//! [`Regexp#match?`](https://ruby-doc.org/core-2.6.3/Regexp.html#method-i-match-3F)

use std::convert::TryFrom;
use std::mem;

use crate::convert::{Convert, RustBackedValue, TryConvert};
use crate::extn::core::regexp::{Backend, Regexp};
use crate::sys;
use crate::types::Int;
use crate::value::Value;
use crate::Artichoke;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Error {
    Fatal,
    PosType,
    StringType,
}

#[derive(Debug)]
pub struct Args {
    pub string: Option<String>,
    pub pos: Option<Int>,
}

impl Args {
    const ARGSPEC: &'static [u8] = b"o|o?\0";

    pub unsafe fn extract(interp: &Artichoke) -> Result<Self, Error> {
        let mrb = interp.0.borrow().mrb;
        let mut string = <mem::MaybeUninit<sys::mrb_value>>::uninit();
        let mut pos = <mem::MaybeUninit<sys::mrb_value>>::uninit();
        let mut has_pos = <mem::MaybeUninit<sys::mrb_bool>>::uninit();
        sys::mrb_get_args(
            mrb,
            Self::ARGSPEC.as_ptr() as *const i8,
            string.as_mut_ptr(),
            pos.as_mut_ptr(),
            has_pos.as_mut_ptr(),
        );
        let string = string.assume_init();
        let has_pos = has_pos.assume_init() != 0;
        let string = if let Ok(string) = interp.try_convert(Value::new(interp, string)) {
            string
        } else {
            return Err(Error::StringType);
        };
        let pos = if has_pos {
            let pos = interp
                .try_convert(Value::new(&interp, pos.assume_init()))
                .map_err(|_| Error::PosType)?;
            Some(pos)
        } else {
            None
        };
        Ok(Self { string, pos })
    }
}

pub fn method(interp: &Artichoke, args: Args, value: &Value) -> Result<Value, Error> {
    let data = unsafe { Regexp::try_from_ruby(interp, value) }.map_err(|_| Error::Fatal)?;
    let string = if let Some(string) = args.string {
        string
    } else {
        return Ok(interp.convert(false));
    };
    let pos = args.pos.unwrap_or_default();
    let pos = if pos < 0 {
        let strlen = Int::try_from(string.chars().count()).unwrap_or_default();
        let pos = strlen + pos;
        if pos < 0 {
            return Ok(interp.convert(false));
        }
        usize::try_from(pos).map_err(|_| Error::Fatal)?
    } else {
        usize::try_from(pos).map_err(|_| Error::Fatal)?
    };
    // onig will panic if pos is beyond the end of string
    if pos > string.chars().count() {
        return Ok(interp.convert(false));
    }
    let byte_offset = string.chars().take(pos).collect::<String>().len();

    let match_target = &string[byte_offset..];
    let borrow = data.borrow();
    let regex = (*borrow.regex).as_ref().ok_or(Error::Fatal)?;
    match regex {
        Backend::Onig(regex) => Ok(interp.convert(regex.find(match_target).is_some())),
        Backend::Rust(_) => unimplemented!("Rust-backed Regexp"),
    }
}
