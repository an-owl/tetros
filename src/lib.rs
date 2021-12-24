#![no_std]
pub const BLOCK_SIZE: usize = 30; //block should always be square
pub const BOARD_WIDTH: usize = BLOCK_SIZE * Board::GAME_WIDTH;
pub const BOARD_HEIGHT: usize = BLOCK_SIZE * Board::GAME_HEIGHT;

extern crate rlibc;
extern crate alloc;
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
        let (x,y) = location;
        let address = (y * self.width) + x;

        self.contents[address] = colour;
        self.update_block(location);
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
    /*
    let block = board.get_block(BlockColour::Grey).unwrap();
    let (mut x,mut y) = board.location;
    x -= BLOCK_SIZE;
    y -= BLOCK_SIZE;

    g.draw_to_buff(block,0,(0,0));

    info!("{},{}",g.get_resolution().0,g.get_resolution().1);
    info!("{},{}",g.get_buff(0).unwrap().resolution().0,g.get_buff(0).unwrap().resolution().1);
    info!("{},{}",block.resolution().0,block.resolution().1);
     */

    board.render_bg(g.mut_get_buff(0).unwrap());

    g.draw(0).unwrap().unwrap(); //should be only call to g.draw during Gameplay
    board.draw(&mut g).unwrap().unwrap(); //do not draw board to stored buffers it will waste time //TODO handle this






    Ok(uefi::Status::SUCCESS.into())
}