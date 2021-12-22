#![no_main]
#![no_std]
#![feature(abi_efiapi)]

//! hi not as in hello but as in human interface

extern crate rlibc;
extern crate alloc;
extern crate log;
extern crate uefi;


use uefi::prelude::*;
use tetros::*;

#[entry]
fn main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    use core::fmt::Write;
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway
    let o = uefi_things::proto::get_proto::<uefi::proto::console::text::Output>(st.boot_services()).unwrap().unwrap();

    {
        let gop = uefi_things::proto::get_proto::<uefi::proto::console::gop::GraphicsOutput>(st.boot_services()).unwrap().unwrap();
        let (width,height) = gop.current_mode_info().resolution();

        if (width < BOARD_WIDTH) || (height < BOARD_HEIGHT){
            writeln!(o,"unsupported resolution requites at least {}x{}",  BOARD_WIDTH,BOARD_HEIGHT).unwrap();
            return Status::UNSUPPORTED
        }

    }


    run(&st).unwrap().unwrap(); //TODO handle this at some point




    Status::SUCCESS
}