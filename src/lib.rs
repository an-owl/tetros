#![no_std]
pub const BLOCK_SIZE: usize = 30; //block should always be square
pub const BOARD_WIDTH: usize = BLOCK_SIZE * Board::GAME_WIDTH;
pub const BOARD_HEIGHT: usize = BLOCK_SIZE * Board::GAME_HEIGHT;

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;

use uefi::Status;
use uefi_things::glib::{Sprite, GraphicsHandle};
use  core::fmt::Debug;
use uefi_things::fs::GetFileStatus;
use alloc::fmt::Write;
use alloc::vec::Vec;


struct Board{
    //location on screen
    location: (usize,usize),
    //size in blocks
    width: usize,
    height: usize,

    contents: Vec<BlockColour>, //contains block colours within game grid
    blocks: Vec<Block>, //contains block data
    sprite: Sprite,
}

impl Board{

    const GAME_HEIGHT: usize = 18;
    const GAME_WIDTH: usize = 10;
    pub fn new(st: &uefi::prelude::SystemTable<uefi::prelude::Boot>, g: &uefi_things::glib::GraphicsHandle) -> Self{
        let location = {
            let (mut width,mut height) = g.get_resolution();
            //get co-ords of board
            width  -= BOARD_WIDTH;
            height -= BOARD_HEIGHT;

            width /= 2;
            height /= 2;

            (width,height)
        };
        let width = Board::GAME_WIDTH;
        let height = Board::GAME_HEIGHT;

        let mut contents = Vec::new();
        contents.resize(width*height,BlockColour::None);

        let mut blocks = Vec::new();

        {
            let fs = uefi_things::proto::get_proto::<uefi::proto::media::fs::SimpleFileSystem>(st.boot_services()).unwrap().unwrap();

            //loop pls
            blocks.push(BlockColour::Red.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Blue.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Green.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Cyan.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Grey.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Yellow.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Orange.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::Purple.get_as_sprite(fs).unwrap().unwrap());
            blocks.push(BlockColour::None.get_as_sprite(fs).unwrap().unwrap());

        }

        let sprite = Sprite::new(width*BLOCK_SIZE,height*BLOCK_SIZE);



        return Self{
            location,
            width,
            height,
            contents,
            blocks,
            sprite,
        }
    }

    fn get_block(&self, colour: BlockColour) -> Option<&Block>{
        for block in &self.blocks{
            if block.colour == colour{
                return Some(block)
            }
        }
        None
    }
    fn render_bg(&self, sprite: &mut Sprite) {
        let block = self.get_block(BlockColour::Grey).expect("unable to find grey block");
        let (mut start_x,mut start_y) = self.location;
        //one block top right of board
        start_x -= BLOCK_SIZE;
        start_y -= BLOCK_SIZE;

        let mut count = 0;
        for row in 0..self.height + 2{
            let y = start_y + (BLOCK_SIZE * row);
            for col in 0..self.width + 2{
                count += 1;

                let x = start_x + (BLOCK_SIZE * col);
                sprite.render_sprite(block,(x,y))

            }
        }
        info!("blocks drawn: {}",count);
    }
    pub fn draw(&self,g: &mut GraphicsHandle,) -> uefi::Result{
        use uefi::proto::console::gop;
        g.gop.blt(gop::BltOp::BufferToVideo {
            buffer: &self.sprite,
            src: gop::BltRegion::Full,
            dest: self.location,
            dims: self.resolution()
        })
    }

    fn update_block(&mut self,location: (usize,usize)){
        let (x,y) = location;
        let address = (y * self.width) + x;
        let colour = self.contents[address];
        let block = self.get_block(colour).unwrap().clone();

        self.sprite.render_sprite(&block,(x*BLOCK_SIZE,y*BLOCK_SIZE));

    }

    fn set_and_update(&mut self,location: (usize,usize),colour: BlockColour){
        self.set(location,colour);
        self.update_block(location);
    }

    fn set(&mut self, at:(usize, usize), colour: BlockColour){
        //if out of bounds
        if (at.0 > self.width) || (at.1 > self.height){ return }

        let (x,y) = at;
        let address = (y * self.width) + x;

        self.contents[address] = colour;
    }

}

impl core::ops::Deref for Board {
    type Target = Sprite;

    fn deref(&self) -> &Self::Target {
        &self.sprite
    }
}

#[derive(Clone)]
struct Block{
    pub colour: BlockColour,
    pub sprite: Sprite,
}

impl core::ops::Deref for Block{
    type Target = Sprite;

    fn deref(&self) -> &Self::Target {
        &self.sprite
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
enum BlockColour{
    Red,
    Blue,
    Green,
    Cyan,
    Grey,
    Yellow,
    Orange,
    Purple,
    None,
}

impl alloc::fmt::Display for BlockColour {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl BlockColour{
    const BLOCK_PATH: &'static str = "/tetros/blocks/";
    fn get_as_sprite(self,fs: &mut uefi::proto::media::fs::SimpleFileSystem) -> uefi::Result<Block>{
        use uefi::proto::media::file::{FileMode, FileAttribute};
        use uefi::proto::media::file::FileType;
        // if none create blank sprite
        /*if let None = self{

            let mut data = Vec::new();
            data.resize(BLOCK_SIZE*BLOCK_SIZE,BltPixel::new(0,0,0));

            let mut s = Sprite::new(BLOCK_SIZE,BLOCK_SIZE);
            *s = *data;

            return Ok(uefi::Completion::new(Status::SUCCESS,
            Block{
                colour: self,
                sprite: s
            }))
        }*/

        let mut f_name = alloc::string::String::new();
        write!(f_name,"{}{:?}.ppm",Self::BLOCK_PATH,self).unwrap();

        let ppm_file = match uefi_things::fs::get_file_from_path(fs,&f_name,FileMode::Read,FileAttribute::empty()){
            GetFileStatus::Found(f) => f.into_type().unwrap().unwrap(),
            GetFileStatus::NotFound(_) => {

                let s = Sprite::new(BLOCK_SIZE,BLOCK_SIZE);
                return Ok(uefi::Completion::new(Status::SUCCESS,
                Block{
                    colour: self,
                    sprite: s
                }))
            },
                GetFileStatus::Err(e) => return Err(e.into()),
        };

        let raw_ppm = match ppm_file{
            FileType::Regular(f) => {
                uefi_things::fs::read_file(f).unwrap().unwrap()
            }
            FileType::Dir(_) => {
                return Err(Status::LOAD_ERROR.into())
            }
        };
        let mut sprite = Sprite::new(BLOCK_SIZE,BLOCK_SIZE);
        if let Err(_) = sprite.read_ppm(&raw_ppm){
            return Err(Status::LOAD_ERROR.into());
        };

        let block = Block{
            colour: self,
            sprite
        };

        return Ok(uefi::Completion::new(Status::SUCCESS, block));
    }
}

struct Tetromino {
    height: usize,
    width: usize,
    location: (usize,usize),
    colour: BlockColour,
    contents: Vec<bool>,
}

impl Tetromino{
    const SQUARE: u16 = 0b1111;
    const T_SHAPE: u16 = 0b010111;
    const L_SHAPE: u16 = 0b101011;
    const L_SHAPE_R: u16 = 0b010111;
    const I_SHAPE: u16 = 0b1111;
    const Z_SHAPE: u16 = 0b110011;
    const Z_SHAPE_R: u16 = 0b011110;
    /// layout as a binary representation of layout where bit order is layout order
    /// excess bits will be ignores
    fn new(size: (usize,usize) ,layout: u16, colour: BlockColour) -> Self{

        let bt = |i: u16, test: u16| -> bool {
            let mut bit = 1;
            bit <<= i;

            return if (bit & test) == 0 {
                false
            } else {
                true
            }

        };

        let (width,height) = size;
        let mut contents = Vec::new();
        contents.resize(width*height,false);

        for i in 0..contents.len(){
            contents[i] = bt(i as u16,layout);
        }
        //location out of bounds will just silent error
        let location = (Board::GAME_WIDTH + 1,Board::GAME_HEIGHT + 1);

        return Self{
            height,
            width,
            location,
            colour,
            contents
        }

    }

    fn locate(&self,index: usize) -> (usize,usize){
        let y = index / self.width;
        let x = index % self.width;
        (x,y)
    }
    fn index(&self,coords: (usize,usize)) -> usize{
        let (x,y) = coords;
        //info!("({} * {}) + {} = {}",y,self.width,x,(y*self.width ) + x);
        (y * self.width) + x
    }

    fn get_scan(&self, y: usize) -> Vec<bool>{
        let mut scan = Vec::new();
        scan.resize(self.width,false);
        for block in 0..self.width{
            scan[block] = self.contents[self.index((block,y))]
        }
        scan
    }

    fn rotate_right(&mut self){
        let mut scratch = Tetromino::new((self.height,self.width),0,self.colour);

        for scan in 0..self.height {
            let scan_dat = self.get_scan(scan);
            for block in 0..self.width {

                let far = scratch.index(((scratch.width - 1) - scan,block));
                scratch.contents[far] = scan_dat[block];

            }
        }

        self.width = scratch.width;
        self.height = scratch.height;
        self.contents = scratch.contents;
    }


    fn rotate_left(&mut self){
        let mut scratch = Tetromino::new((self.height,self.width),0,self.colour);

        for scan in 0..self.height {
            let scan_dat = self.get_scan(scan);
            for block in 0..self.width {
                //info!("from {:?} index {}",() )
                let far = scratch.index((scan,(scratch.height - 1) - block));
                scratch.contents[far] = scan_dat[block];

            }
        }

        self.width = scratch.width;
        self.height = scratch.height;
        self.contents = scratch.contents;
    }

    fn set(&self, board: &mut Board){
        for i in 0..self.contents.len(){
            if self.contents[i] == false{
                continue
            }
            let (mut x ,mut y) = self.locate(i);
            x += self.location.0;
            y += self.location.1;

            board.set_and_update((x,y),self.colour);
        }
    }
    fn unset(&self, board: &mut Board){
        for i in 0..self.contents.len(){
            let (mut x ,mut y) = self.locate(i);
            x += self.location.0;
            y += self.location.1;

            board.set_and_update((x,y),BlockColour::None);
        }
    }
}


pub fn run(st: &uefi::table::SystemTable<uefi::prelude::Boot>) -> uefi::Result<()>{
    // Get required protocols
    use uefi_things::proto::get_proto;
    use uefi::proto::console::text::Output;
    use uefi::proto::console::gop::GraphicsOutput;

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

    let mut l_shape = Tetromino::new((2,3),Tetromino::L_SHAPE,BlockColour::Blue);
    l_shape.location = (3,3);

    l_shape.set(&mut board);
    board.draw(&mut g).unwrap().unwrap();
    for _ in 0..100 {
        st.boot_services().stall(1000000);
        l_shape.unset(&mut board);
        l_shape.rotate_left();
        l_shape.set(&mut board);
        board.draw(&mut g);
    }



    Ok(uefi::Status::SUCCESS.into())
}