#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]

#![cfg_attr(target_arch = "aarch64", no_std)]

#[cfg(not(target_arch = "aarch64"))]
extern crate core;
extern crate volatile;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod common;
pub mod atags;
pub mod interrupt;
