#![no_std]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

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
    st.boot_services().stall(1000000);

    for _ in 0..5 {
        //info!("moved {} to {:?}",l_shape.safe_ror(&mut board),l_shape.location);
        info!("moved {} to {:?}",l_shape.legal_move((-1,0),&mut board),l_shape.location);
        board.draw(&mut g);
        st.boot_services().stall(1000000);
    }
    info!("moved {} to {:?}",l_shape.legal_move((2,0),&mut board),l_shape.location);
    for _ in 0..5 {
        info!("moved {} to {:?}",l_shape.safe_ror(&mut board),l_shape.location);
        //info!("moved {} to {:?}",l_shape.legal_move((-1,0),&mut board),l_shape.location);
        board.draw(&mut g);
        st.boot_services().stall(1000000);
    }
        for _ in 0..4 {
        info!("moved {} to {:?}",l_shape.safe_rol(&mut board),l_shape.location);
        //info!("moved {} to {:?}",l_shape.legal_move((-1,0),&mut board),l_shape.location);
        board.draw(&mut g);
        st.boot_services().stall(1000000);
    }

    for _ in 0..10 {
        //info!("moved {} to {:?}",l_shape.safe_ror(&mut board),l_shape.location);
        info!("moved {} to {:?}",l_shape.legal_move((1,0),&mut board),l_shape.location);
        board.draw(&mut g);
        st.boot_services().stall(1000000);
    }
    Ok(uefi::Status::SUCCESS.into())
}