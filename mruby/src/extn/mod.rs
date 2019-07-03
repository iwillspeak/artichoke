use crate::interpreter::{Mrb, MrbApi};
use crate::sys;
use crate::MrbError;

pub mod core;
pub mod stdlib;

pub const RUBY_PLATFORM: &str = "x86_64-unknown-mruby";
pub const INPUT_RECORD_SEPARATOR: &str = "\n";

pub fn patch(interp: &Mrb) -> Result<(), MrbError> {
    let mrb = interp.borrow().mrb;
    unsafe {
        let ruby_platform = interp.string(RUBY_PLATFORM);
        sys::mrb_define_global_const(
            mrb,
            b"RUBY_PLATFORM\0".as_ptr() as *const i8,
            ruby_platform.inner(),
        );
        let ruby_description = interp.string(sys::mruby_sys_version(true));
        sys::mrb_define_global_const(
            mrb,
            b"RUBY_DESCRIPTION\0".as_ptr() as *const i8,
            ruby_description.inner(),
        );
        let input_record_separator = interp.string(INPUT_RECORD_SEPARATOR);
        sys::mrb_gv_set(
            mrb,
            interp.borrow_mut().sym_intern("$/"),
            input_record_separator.inner(),
        );
    }
    core::patch(interp)?;
    stdlib::patch(interp)?;
    Ok(())
}