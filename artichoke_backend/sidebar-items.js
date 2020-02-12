initSidebarItems({"fn":[["interpreter","Create and initialize an [`Artichoke`] interpreter."]],"macro":[["mrb_get_args","Extract [`sys::mrb_value`]s from a [`sys::mrb_state`] to adapt a C entrypoint to a Rust implementation of a Ruby function."],["unwrap_interpreter","Extract an `Artichoke` instance from the userdata on a [`sys::mrb_state`]."]],"mod":[["class",""],["convert",""],["def",""],["exception",""],["exception_handler",""],["extn",""],["ffi","Functions for interacting directly with mruby structs from [`sys`]."],["fs","Virtual filesystem."],["gc",""],["method",""],["module",""],["state",""],["string","Utilities for working with Ruby `String`s."],["sys","Rust bindings for mruby, customized for Artichoke."],["types",""],["value",""]],"struct":[["Artichoke","Interpreter instance."]],"trait":[["Convert","Infallible conversion between two types."],["ConvertMut","Mutable infallible conversion between two types."],["Eval","Execute code and retrieve its result."],["File","Rust extension hook that can be required."],["Intern","Store and retrieve byte vectors that have the same lifetime as the interpreter."],["LoadSources","Load Ruby sources and Rust extensions into an interpreter."],["Parser","Manage parser state, active filename context, and line number metadata."],["TopSelf","Return a `Value`-wrapped reference to [top self][topself]."],["TryConvert","Fallible conversions between two types."],["TryConvertMut","Mutable fallible conversions between two types."],["ValueLike","A boxed Ruby value owned by the interpreter."],["Warn","Emit warnings during interpreter execution to stderr."]]});