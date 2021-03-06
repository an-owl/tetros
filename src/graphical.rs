use uefi::Status;
use uefi_things::glib::{Sprite, GraphicsHandle};
use  core::fmt::Debug;
use uefi_things::fs::GetFileStatus;
use alloc::fmt::Write;
use alloc::vec::Vec;

pub const BLOCK_SIZE: usize = 30; //block should always be square
pub const BOARD_WIDTH: usize = BLOCK_SIZE * Board::GAME_WIDTH;
pub const BOARD_HEIGHT: usize = BLOCK_SIZE * Board::GAME_HEIGHT;



pub struct Board{
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
    pub fn render_bg(&self, sprite: &mut Sprite) {
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

    pub fn update_block(&mut self,location: (usize,usize)){
        let (x,y) = location;
        let address = (y * self.width) + x;
        let colour = self.contents[address];
        let block = self.get_block(colour).unwrap().clone();

        self.sprite.render_sprite(&block,(x*BLOCK_SIZE,y*BLOCK_SIZE));

    }

    pub fn set_and_update(&mut self,location: (usize,usize),colour: BlockColour){
        self.set(location,colour);
        self.update_block(location);
    }

    pub fn set(&mut self, at:(usize, usize), colour: BlockColour){
        //if out of bounds
        if (at.0 > self.width) || (at.1 > self.height){ return }

        let (x,y) = at;
        let address = (y * self.width) + x;

        self.contents[address] = colour;


    }

    fn is_free(&self,coords: (usize,usize)) -> bool{
        let (x,y) = coords;
        //info!("({} * {}) + {} = {}",y,self.width,x,(y*self.width ) + x);
        let index = (y * self.width) + x;

        if x > self.width  { return false }
        if y > self.height { return false }

        return if self.contents[index] == BlockColour::None {
            true
        } else {
            false
        }
    }

    fn index(&self,coords:(usize,usize)) -> usize {
        (coords.1 * self.width) + coords.0
    }

    /// checks rows top..bottom returns row of first full row
    fn find_row(&self) -> Option<usize>{
        //iterates through rows
        'row: for row in  0..self.height {

            for block in 0..self.width {
                //checks for BlockColour::None
                //if found row is not full so it continues to the next row
                if let BlockColour::None = self.contents[self.index((block,row))] {
                    continue 'row;
                }
            }
            //this is only reached when the above loop completes
            //it should only complete if the row is full
            return Some(row);
        }
        None
    }

    /// clears given row
    fn clear_row(&mut self, row: usize){
        for i in 0..self.width{
            self.set_and_update((i,row),BlockColour::None);
        }
    }

    ///scrolls all blocks above `row` down
    fn scroll_from(&mut self, row: usize){
        for r in (1..row+1).rev(){
            for block in 0..self.width{
                let state = self.contents[self.index((block,r-1))];
                self.set_and_update((block,r) ,state);
            }
        }
        self.clear_row(0);
    }

    ///clears full rows and scrolls blocks down
    /// returns number of rows cleared
    pub fn clean_screen(&mut self) -> i64{ //return type subject to change
        let mut cleared = 0;

        loop {
            match self.find_row(){
                None => break,
                Some( row ) => {
                    cleared += 1;
                    self.clear_row(row);
                    self.scroll_from(row);
                }
            }
        }
        return cleared
    }

    pub fn is_lost(&self) -> bool{
        for block in 0..self.width{
            if !self.is_free((block,0)){
                return true
            }
        }
        false
    }
}

impl core::ops::Deref for Board {
    type Target = Sprite;

    fn deref(&self) -> &Self::Target {
        &self.sprite
    }
}

#[derive(Clone)]
pub struct Block{
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
pub enum BlockColour{
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
    pub fn get_as_sprite(self,fs: &mut uefi::proto::media::fs::SimpleFileSystem) -> uefi::Result<Block>{
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

pub struct Tetromino {
    height: usize,
    width: usize,
    pub location: (isize,isize),
    pub colour: BlockColour,
    contents: Vec<bool>,
}

impl Tetromino{
    pub const SQUARE: u16 = 0b1111;
    pub const T_SHAPE: u16 = 0b010111000;
    pub const L_SHAPE: u16 = 0b01001001011;
    pub const L_SHAPE_R: u16 = 0b010010110;
    pub const I_SHAPE: u16 = 0b1111;
    pub const Z_SHAPE: u16 = 0b110011000;
    pub const Z_SHAPE_R: u16 = 0b000011110;
    /// layout as a binary representation of layout where bit order is layout order
    /// excess bits will be ignores
    pub fn new(size: (usize,usize) ,layout: u16, colour: BlockColour) -> Self{

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
        let location = (0,0);
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

    pub fn rotate_right(&mut self){
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


    pub fn rotate_left(&mut self){
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

    pub fn set(&self, board: &mut Board){
        for i in 0..self.contents.len(){
            if self.contents[i] == false{
                continue
            }
            let (x ,y) = self.locate(i);
            let mut x = x as isize;
            let mut y = y as isize;

            x += self.location.0;
            y += self.location.1;

            if x.is_negative() || y.is_negative() { continue }

            board.set_and_update((x as usize,y as usize),self.colour);
        }
    }
    pub fn unset(&self, board: &mut Board){
        for i in 0..self.contents.len(){
            if !self.contents[i] { continue }
            let (x  ,y) = self.locate(i);
            let mut x = x as isize;
            let mut y = y as isize;

            x += self.location.0;
            y += self.location.1;

            if x.is_negative() || y.is_negative() {
                continue
            }

            board.set_and_update((x as usize,y as usize),BlockColour::None);
        }
    }
    pub fn do_and_update(&mut self,task: fn(&mut Self), board: &mut Board, g: &mut GraphicsHandle){
        self.unset(board);
        task(self);
        self.set(board);
        board.draw(g).unwrap().unwrap();
    }

    pub fn relocate(&mut self, to: (i8,i8)) -> Result<(),()> {
        let (x,y) = to;

        self.location.0 += x as isize;
        self.location.1 += y as isize;

        return Ok(())
    }

    /// check for occupied spaces around tetromino
    /// returns true if self can stay here

    pub fn is_legal(&self, board: &Board) -> bool{
        //check boundaries
        for i in 0..self.contents.len(){
            if self.contents[i] == false {continue}

            let (x,y) = self.locate(i);
            let mut x = x as isize;
            let mut y = y as isize;

            //info!("x: {} + {}",x,self.location.0);
            //info!("y: {} + {}",y,self.location.1);

            x += self.location.0;
            y += self.location.1;

            if x.is_negative() || y.is_negative(){
                return false
            }
            if (x as usize >= board.width ) || (y as usize >= board.height) {
                return false
            }
            if !board.is_free((x as usize,y as usize)){
                return false
            }
        }
        true
    }

    /// checks left and right of current tetromino for legal spaces
    /// returns `(left,below,right)`
    /// self must be unset beforehand
    pub fn check_sides(&self, board: &Board) -> (bool,bool,bool) {
        let mut test = Tetromino{
            height: self.height,
            width: self.height,
            location: self.location,
            colour: self.colour,
            contents: self.contents.clone()
        };

        test.relocate((-1,0)).unwrap();
        let left = test.is_legal(board);

        test.relocate((2,0)).unwrap();
        let right = test.is_legal(board);

        test.relocate((-1,1)).unwrap();
        let below = test.is_legal(board);

        return (left,below,right)

    }

    pub fn legal_move(&mut self, to: (i8,i8),board: &mut Board) -> bool{
        self.unset(board);
        if let Err(_) = self.relocate(to) {
            self.set(board);
            return false
        };
        let mut ret = true;
        if !self.is_legal(board){
            let (x,y) = to;
            self.relocate((-x,-y)).unwrap();
            ret = false;
        }


        self.set(board);
        ret
    }

    pub fn safe_ror(&mut self, board: &mut Board) -> bool{
        self.unset(board);
        self.rotate_right();
        if !self.is_legal(board){
            let (left,_,right) = self.check_sides(board);
            if left { self.relocate((-1,0)).unwrap();
            } else if right { self.relocate((1,0)).unwrap();
            } else {
                self.rotate_left();
                self.set(board);
                return false
            }
        }
        self.set(board);
        true
    }
    pub fn safe_rol(&mut self, board: &mut Board) -> bool{
        self.unset(board);
        self.rotate_left();
        if !self.is_legal(board) {
            let (left,_,right) = self.check_sides(board);
            if left { self.relocate((-1,0)).unwrap();
            } else if right { self.relocate((1,0)).unwrap();
            } else {
                self.rotate_left();
                self.set(board);
                return false
            }
        }
        self.set(board);
        true
    }

}