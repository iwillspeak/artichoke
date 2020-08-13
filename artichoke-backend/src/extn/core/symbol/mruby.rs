use crate::extn::core::symbol::{self, trampoline};
use crate::extn::prelude::*;

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    if interp.is_class_defined::<symbol::Symbol>() {
        return Ok(());
    }
    let spec = class::Spec::new("Symbol", None, None)?;
    class::Builder::for_spec(interp, &spec)
        .add_self_method(
            "all_symbols",
            artichoke_symbol_all_symbols,
            sys::mrb_args_none(),
        )?
        .add_method("==", artichoke_symbol_equal_equal, sys::mrb_args_req(1))?
        .add_method(
            "casecmp",
            artichoke_symbol_ascii_casecmp,
            sys::mrb_args_req(1),
        )?
        .add_method(
            "casecmp?",
            artichoke_symbol_unicode_casecmp,
            sys::mrb_args_req(1),
        )?
        .add_method("empty?", artichoke_symbol_empty, sys::mrb_args_none())?
        .add_method("inspect", artichoke_symbol_inspect, sys::mrb_args_none())?
        .add_method("length", artichoke_symbol_length, sys::mrb_args_none())?
        .add_method("to_s", artichoke_symbol_to_s, sys::mrb_args_none())?
        .define()?;
    interp.def_class::<symbol::Symbol>(spec)?;
    let _ = interp.eval(&include_bytes!("symbol.rb")[..])?;
    trace!("Patched Symbol onto interpreter");
    Ok(())
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_all_symbols(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let result = trampoline::all_symbols(&mut guard);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_equal_equal(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    let other = mrb_get_args!(mrb, required = 1);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let other = Value::from(other);
    let result = trampoline::equal_equal(&mut guard, sym, other);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_ascii_casecmp(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    let other = mrb_get_args!(mrb, required = 1);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let other = Value::from(other);
    let result = trampoline::ascii_casecmp(&mut guard, sym, other);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_unicode_casecmp(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    let other = mrb_get_args!(mrb, required = 1);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let other = Value::from(other);
    let result = trampoline::unicode_casecmp(&mut guard, sym, other);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_empty(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let result = trampoline::is_empty(&mut guard, sym);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_inspect(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let value = Value::from(slf);
    let result = trampoline::inspect(&mut guard, value);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_length(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let result = trampoline::length(&mut guard, sym);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_symbol_to_s(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let mut guard = Guard::new(&mut interp);
    let sym = Value::from(slf);
    let result = trampoline::bytes(&mut guard, sym);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}