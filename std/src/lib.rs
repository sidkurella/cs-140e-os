#![stable(feature = "rust1", since = "1.0.0")]

// Don't link to std. We are std.
#![no_std]

// std is implemented with unstable features, many of which are internal
// compiler details that will never be stable
// #![feature(alloc)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(allocator_internals)]
#![feature(allow_internal_unsafe)]
#![feature(allow_internal_unstable)]
#![feature(align_offset)]
#![feature(arbitrary_self_types)]
#![feature(array_error_internals)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(c_variadic)]
#![feature(cfg_target_has_atomic)]
#![feature(cfg_target_thread_local)]
#![feature(char_error_internals)]
#![feature(compiler_builtins_lib)]
#![feature(concat_idents)]
#![feature(const_raw_ptr_deref)]
// #![feature(const_cstr_unchecked)]
#![feature(core_intrinsics)]
#![feature(core_panic_info)]
#![feature(dropck_eyepatch)]
#![feature(duration_constants)]
#![feature(exact_size_is_empty)]
#![feature(external_doc)]
#![feature(fixed_size_array)]
#![feature(fn_traits)]
// #![feature(fnbox)]
#![feature(futures_api)]
#![feature(generator_trait)]
#![feature(hashmap_internals)]
#![feature(int_error_internals)]
// #![feature(integer_atomics)]
#![feature(lang_items)]
#![feature(libc)]
#![feature(link_args)]
#![feature(linkage)]
#![feature(needs_panic_runtime)]
#![feature(never_type)]
#![feature(nll)]
#![feature(exhaustive_patterns)]
#![feature(on_unimplemented)]
#![feature(optin_builtin_traits)]
#![feature(panic_internals)]
// #![feature(panic_unwind)]
#![feature(prelude_import)]
#![feature(ptr_internals)]
#![feature(raw)]
// #![feature(hash_raw_entry)]
#![feature(rustc_attrs)]
#![feature(rustc_const_unstable)]
#![feature(std_internals)]
#![feature(stdsimd)]
// #![feature(shrink_to)]
// #![feature(slice_concat_ext)]
#![feature(slice_internals)]
#![feature(slice_patterns)]
#![feature(staged_api)]
#![feature(stmt_expr_attributes)]
#![feature(str_internals)]
#![feature(renamed_spin_loop)]
#![feature(rustc_private)]
#![feature(thread_local)]
// #![feature(toowned_clone_into)]
#![feature(try_from)]
// #![feature(try_reserve)]
#![feature(unboxed_closures)]
#![feature(untagged_unions)]
#![feature(unwind_attributes)]
#![feature(doc_cfg)]
#![feature(doc_masked)]
#![feature(doc_spotlight)]
#![feature(doc_alias)]
#![feature(doc_keyword)]
#![feature(panic_info_message)]
#![feature(non_exhaustive)]
#![feature(alloc_layout_extra)]
#![feature(maybe_uninit)]

// Explicitly import the prelude. The compiler uses this same unstable attribute
// to import the prelude implicitly when building crates that depend on std.
#[prelude_import]
#[allow(unused)]
use prelude::v1::*;

// // Access to Bencher, etc.
// #[cfg(test)] extern crate test;
// #[cfg(test)] extern crate rand;

// Re-export a few macros from core
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::{assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::{unreachable, unimplemented, panic, write, writeln, try};

// #[macro_use]
// #[macro_reexport(vec, format)]
// extern crate alloc;
// extern crate alloc_system;
// extern crate std_unicode;
// #[doc(masked)]
// extern crate libc;

// // We always need an unwinder currently for backtraces
// #[doc(masked)]
// #[allow(unused_extern_crates)]
// extern crate unwind;

// compiler-rt intrinsics
// #[doc(masked)]
// extern crate compiler_builtins;

// // During testing, this crate is not actually the "real" std library, but rather
// // it links to the real std library, which was compiled from this same source
// // code. So any lang items std defines are conditionally excluded (or else they
// // wolud generate duplicate lang item errors), and any globals it defines are
// // _not_ the globals used by "real" std. So this import, defined only during
// // testing gives test-std access to real-std lang items and globals. See #2912
// #[cfg(test)] extern crate std as realstd;

// The standard macros that are not built-in to the compiler.
#[macro_use]
mod macros;

// The Rust prelude
pub mod prelude;

// Public module declarations and re-exports
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::any;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::cell;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::clone;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::cmp;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::convert;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::default;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::hash;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::intrinsics;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::iter;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::marker;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::mem;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::ops;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::ptr;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::raw;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::result;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::option;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::isize;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i8;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i16;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i32;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i64;
#[unstable(feature = "i128", issue = "35118")]
pub use core::i128;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::usize;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u8;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u16;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u32;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u64;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::boxed;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::rc;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::borrow;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::fmt;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::fmt;
#[stable(feature = "pin", since = "1.33.0")]
pub use core::pin;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::str;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::string;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::vec;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::char;
#[unstable(feature = "i128", issue = "35118")]
pub use core::u128;

// TODO: This is an addition. This should should actually come from `alloc`.
// pub mod str;
// pub mod slice;

// pub mod f32;
// pub mod f64;

// #[macro_use]
// pub mod thread;
// pub mod ascii;
// pub mod collections;
// pub mod env;
// pub mod error;
// pub mod ffi;
// pub mod fs;
pub mod io;
// pub mod net;
// pub mod num;
// pub mod os;
// pub mod panic;
// pub mod path;
// pub mod process;
pub mod sync;
// pub mod time;
// pub mod heap;

// // Platform-abstraction modules
// #[macro_use]
// mod sys_common;
// mod sys;

// // Private support modules
// mod panicking;
// mod memchr;

// // The runtime entry point and a few unstable public functions used by the
// // compiler
// pub mod rt;
// // The trait to support returning arbitrary types in the main function
// mod termination;

// #[unstable(feature = "termination_trait", issue = "43301")]
// pub use self::termination::Termination;
