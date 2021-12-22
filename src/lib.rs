#![no_std]
pub const BLOCK_SIZE: usize = 30; //block should always be square
pub const BOARD_WIDTH: usize = BLOCK_SIZE * 12;
pub const BOARD_HEIGHT: usize = BLOCK_SIZE * 20;

extern crate rlibc;
extern crate alloc;
extern crate log;
extern crate uefi;

use uefi::Status;
use uefi_things::glib::Sprite;
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
        let height = Board::GAME_WIDTH;

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

        let sprite = Sprite::new((width*BLOCK_SIZE),(height*BLOCK_SIZE));



        return Self{
            location,
            width,
            height,
            contents,
            blocks,
            sprite,
        }
    }
}

struct Block{
    pub colour: BlockColour,
    pub sprite: Sprite,
}

#[derive(Debug,Clone,Copy)]
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
        use uefi::proto::console::gop::BltPixel;
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
                let mut data = Vec::new();
                data.resize(BLOCK_SIZE*BLOCK_SIZE,BltPixel::new(0,0,0));

                let mut s = Sprite::new(BLOCK_SIZE,BLOCK_SIZE);
                if s.len() == data.len(){
                    s.copy_from_slice(&data)
                }


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
        sprite.read_ppm(&raw_ppm);

        let block = Block{
            colour: self,
            sprite
        };

        return Ok(uefi::Completion::new(Status::SUCCESS, block));
    }
}

pub fn run(st: &uefi::table::SystemTable<uefi::prelude::Boot>) -> Status{
    // Get required protocols
    use uefi_things::proto::get_proto;
    use uefi::proto::console::text::Output;
    use uefi::proto::console::gop::GraphicsOutput;
    use uefi::proto::media::fs::SimpleFileSystem;

    let mut o = get_proto::<Output>(st.boot_services()).unwrap().unwrap();
    let g = uefi_things::glib::GraphicsHandle::new(
        uefi_things::proto::get_proto::<GraphicsOutput>(st.boot_services()).unwrap().unwrap(),
    None,
    );

    //check resolution


    let board = Board::new(st,&g);


    uefi::Status::SUCCESS
}