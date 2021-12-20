#![no_main]
#![no_std]
#![feature(abi_efiapi)]

//! hi not as in hello but as in human interface

extern crate rlibc;
extern crate alloc;
extern crate log;
extern crate uefi;


use uefi::prelude::*;

#[entry]
fn main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway




    Status::SUCCESS
}