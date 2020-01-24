#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(pointer_methods)]

#![cfg_attr(target_arch = "aarch64", no_std)]

#[cfg(not(target_arch = "aarch64"))]
extern crate core;
extern crate volatile;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod common;
pub mod atags;
