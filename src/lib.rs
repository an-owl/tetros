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


    log::set_max_level(log::LevelFilter::Debug);
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

    let mut tetrominos = alloc::vec::Vec::new();

    {
        let l_shape = Tetromino::new((3,3),Tetromino::L_SHAPE,BlockColour::Blue);
        let square = Tetromino::new((2,2),Tetromino::SQUARE,BlockColour::Yellow);
        let j_shape = Tetromino::new((3,3),Tetromino::L_SHAPE_R,BlockColour::Red);
        let z_shape = Tetromino::new((3,3), Tetromino::Z_SHAPE,BlockColour::Green);
        let s_shape = Tetromino::new((3,3),Tetromino::Z_SHAPE_R,BlockColour::Orange);
        let i_shape = Tetromino::new((4,1),Tetromino::I_SHAPE,BlockColour::Cyan);
        let t_shape = Tetromino::new((3,3),Tetromino::T_SHAPE,BlockColour::Purple);

        tetrominos.push(l_shape);
        tetrominos.push(square);
        tetrominos.push(t_shape);
        tetrominos.push(j_shape);
        tetrominos.push(z_shape);
        tetrominos.push(s_shape);
        tetrominos.push(i_shape);

    }

    /*
    let mut l_shape = Tetromino::new((3,3),Tetromino::L_SHAPE,BlockColour::Blue);
    let mut square = Tetromino::new((2,2),Tetromino::SQUARE,BlockColour::Yellow);

    l_shape.location = (3,3);
    square.location = (5,3);
    //square.set(&mut board);
    */

    //main game loop
    loop {
        tetrominos[0].location = (3,0);
        tetrominos[0].set(&mut board);
        board.draw(&mut g).unwrap().unwrap();
        let game_action= |key| -> bool {do_game_action(&mut tetrominos[0], &mut board, key, &mut g)}; //TODO get random tetromino
        if tick(st, 7_000, game_action) { break }

    }
    uefi_things::proto::get_proto::<Output>(st.boot_services()).unwrap().unwrap().clear().unwrap().unwrap();
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
                    tet.safe_ror(board);
                    board.draw(g).unwrap().unwrap();
                }
                'q' => {
                    trace!("got q");
                    tet.safe_rol(board);
                    board.draw(g).unwrap().unwrap();
                }

                // left right movement
                'a' => {
                    trace!("Go right");
                    tet.legal_move((-1,0),board);
                    board.draw(g).unwrap().unwrap();
                }
                'd' => {
                    trace!("Go left");
                    tet.legal_move((1,0), board);
                    board.draw(g).unwrap().unwrap();
                }

                //fast drop
                's' => {
                    trace!("dropping");
                    tet.legal_move((0,1),board);
                    board.draw(g).unwrap().unwrap();
                }
                'w' => {}

                e => {trace!("got something {}",e);} //do nothing
            }
        }
        uefi::proto::console::text::Key::Special(uefi::proto::console::text::ScanCode::ESCAPE) => {
            board.clean_screen();
            board.draw(g).unwrap().unwrap();
            return true
        } //pause
        _ => {}
    }
    return false
}