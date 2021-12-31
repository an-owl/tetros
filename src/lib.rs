#![no_std]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use uefi::prelude::*;
use crate::graphical::*;
use uefi_things::glib::GraphicsHandle;


pub mod graphical;


pub fn run(st: &uefi::table::SystemTable<uefi::prelude::Boot>) -> uefi::Result<()>{
    // Get required protocols
    use uefi_things::proto::get_proto;
    use uefi::proto::console::text::Output;
    use uefi::proto::console::gop::GraphicsOutput;
    use graphical::*;

    // initialize protocols
    let _o = get_proto::<Output>(st.boot_services()).unwrap().unwrap();
    let mut g = uefi_things::glib::GraphicsHandle::new(
        uefi_things::proto::get_proto::<GraphicsOutput>(st.boot_services()).unwrap().unwrap(),
    None,
    );
    //create game board
    let mut board = Board::new(st, &g);
    g.new_buff();
    //create game boarder

    board.render_bg(g.mut_get_buff(0).unwrap());

    g.draw(0).unwrap().unwrap(); //should be only call to g.draw during Gameplay
    board.draw(&mut g).unwrap().unwrap(); //do not draw board to stored buffers it will waste time //TODO handle this

    let mut l_shape = Tetromino::new((3,3),Tetromino::L_SHAPE,BlockColour::Blue);
    let mut square = Tetromino::new((2,2),Tetromino::SQUARE,BlockColour::Yellow);

    l_shape.location = (3,3);
    square.location = (5,3);
    //square.set(&mut board);

    l_shape.set(&mut board);
    board.draw(&mut g);


    //main game loop
    loop {

        let game_action= |key| -> bool {do_game_action(&mut l_shape, &mut board, key, &mut g)};
        if tick(st, 1_000, game_action) { break }

    }
    Ok(uefi::Status::SUCCESS.into())
}


pub fn tick<T>(st: &SystemTable<Boot>,time: u64 , mut action: T) -> bool
    where T: FnMut(uefi::proto::console::text::Key) -> bool
{
    use uefi::table::boot;
    use uefi::ResultExt;
    const MILLI: u64 = 10_000;
    let kb = uefi_things::proto::get_proto::<uefi::proto::console::text::Input>(st.boot_services()).unwrap().unwrap();

    let tick = unsafe {st.boot_services().create_event( boot::EventType::TIMER,
                                                        boot::Tpl::APPLICATION,
                                                        None,
                                                        None
    ).expect_success("Failed to create timer event.")};

    st.boot_services().set_timer(&tick, boot::TimerTrigger::Relative(time*MILLI)).expect_success("Failed to set timer.");

    while !st.boot_services().check_event(unsafe {tick.unsafe_clone()}).unwrap().unwrap(){
        let timeout = unsafe {st.boot_services().create_event( boot::EventType::TIMER,
                                                        boot::Tpl::APPLICATION,
                                                        None,
                                                        None
        ).expect_success("Failed to create timer event.")};
        st.boot_services().set_timer(&timeout ,boot::TimerTrigger::Relative(MILLI)).expect_success("Failed to set timer");
        let key_event = unsafe{ kb.wait_for_key_event().unsafe_clone() };

        if let 0 = st.boot_services().wait_for_event(&mut [timeout,key_event]).expect_success("Failed to wait for key event"){
            if let Some(k) = kb.read_key().expect_success("Failed to get key."){

                if action(k) {return true}

            }
        }
    }
    return false
}



fn do_game_action(tet: &mut Tetromino,board: &mut Board ,key: uefi::proto::console::text::Key ,g: &mut GraphicsHandle) -> bool{

    match key {
        uefi::proto::console::text::Key::Printable(key) => {
            let key = key.into();
            match key{
                // rotation

                'e' => {
                    trace!("got e");
                    debug!("{}",tet.safe_ror(board));
                    board.draw(g);
                }
                'q' => {
                    trace!("got q");
                    debug!("{}",tet.safe_rol(board));
                    board.draw(g);
                }

                // left right movement
                'a' => {
                    trace!("got a");
                    debug!("{}",tet.legal_move((-1,0),board));
                    board.draw(g);
                }
                'd' => {
                    trace!("got d");
                    debug!("{}",tet.legal_move((1,0), board));
                    board.draw(g);
                }

                //fast drop
                's' => {
                    trace!("got s");
                }
                'w' => {
                    trace!("got w");
                }

                e => {trace!("got something {}",e);} //do nothing
            }
        }
        uefi::proto::console::text::Key::Special(uefi::proto::console::text::ScanCode::ESCAPE) => {return true} //pause
        _ => {}
    }
    return false
}