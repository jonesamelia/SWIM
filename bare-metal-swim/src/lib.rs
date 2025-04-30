#![no_std]

use file_system_template::FileSystem;
use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};
use ramdisk::RamDisk;

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive, str,
};


const TASK_MANAGER_WIDTH: usize = 10;
const WIN_REGION_WIDTH: usize = BUFFER_WIDTH - TASK_MANAGER_WIDTH;
const MAX_OPEN: usize = 16;
const BLOCK_SIZE: usize = 256;
const NUM_BLOCKS: usize = 255;
const MAX_FILE_BLOCKS: usize = 64;
const MAX_FILE_BYTES: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;
const MAX_FILES_STORED: usize = 30;
const MAX_FILENAME_BYTES: usize = 10;
const WIN_WIDTH: usize = (WIN_REGION_WIDTH - 3) / 2;



pub struct SwimInterface {
    text: [[char; 38]; 10],
    num_letters: usize,
    next_letter: usize,
    cursorx: usize,
    cursory: usize,
    startx: usize,
    starty: usize,
    height: usize,
    width: usize,
    id: char,
    init: bool,
    active: bool,
    file_sys: FileSystem<MAX_OPEN, BLOCK_SIZE, NUM_BLOCKS, MAX_FILE_BLOCKS, MAX_FILE_BYTES, MAX_FILES_STORED, MAX_FILENAME_BYTES>,
    displaying_files: bool,
    active_file: usize
}


pub struct Swim{
    windows: [SwimInterface; 4],
    init: bool,
    active_win: usize,
    displaying_files: bool
}

impl Default for Swim{
    fn default() -> Self{
        Self{
            windows: [SwimInterface::default(), SwimInterface::default(), SwimInterface::default(), SwimInterface::default()],
            init: false,
            active_win: 0,
            displaying_files: true
        }
    }
}

pub fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

pub fn add1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, 1)
}

pub fn sub1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, LIMIT - 1)
}

impl Default for SwimInterface {
    fn default() -> Self {
        Self {
            text: [[' '; 38]; 10],
            num_letters: 1,
            next_letter: 1,
            cursorx: 1,
            cursory: 2,
            startx: 1,
            starty: 2,
            width: 38,
            height: 10,
            id: ' ',
            init: false,
            active: false,
            file_sys: FileSystem::new(RamDisk::new()),
            displaying_files: true,
            active_file: 0

        }
    }
}

impl Swim{
    pub fn tick(&mut self){
        if self.init == false{
            self.initialize();
        }
        if self.displaying_files{
            self.windows[self.active_win].display_files(); 
        }
        else{
            self.windows[self.active_win].show_cursor();     
        }
        
    }

    fn initialize(&mut self){
        plot_str("Header", 0, 0, ColorCode::new(Color::Cyan, Color::Black));
        self.windows[0].active = true;
        self.windows[0].initialize('1');
        self.windows[1].startx = 40;
        self.windows[1].starty = 2;
        self.windows[1].initialize('2');
        self.windows[2].starty = 13;
        self.windows[2].initialize('3');
        self.windows[3].startx = 40;
        self.windows[3].starty = 13;
        self.windows[3].initialize('4');
        self.windows[0].active_border();
        self.active_win = 0;
        self.init = true;

    }

    fn set_active(&mut self, win: usize){
        self.windows[self.active_win].normal_border();
        self.windows[self.active_win].active = false;
        self.active_win = win;
        self.windows[win].active_border();
    }
    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }
    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => {
                if self.active_win != 0{
                    self.set_active(0);
                }
            }
            KeyCode::F2 => {
                if self.active_win != 1{
                    self.set_active(1);
                }
            }
            KeyCode::F3 => {
                if self.active_win != 2{
                    self.set_active(2);
                }
            }
            KeyCode::F4 => {
                if self.active_win != 3{
                    self.set_active(3);
                }
            }
            KeyCode::ArrowLeft => {
                self.windows[self.active_win].change_active_file(false);
            }
            KeyCode::ArrowRight => {
                self.windows[self.active_win].change_active_file(true);
            }
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        match key{
            '\n' =>{
                if !self.displaying_files{
                    self.windows[self.active_win].enter();    
                }
                
                
            }
            _ =>{
                if is_drawable(key) {
                    if !self.displaying_files{
                        self.windows[self.active_win].add_letter(key);    
                    }
                    
                }    
            }
        }
    }
}


impl SwimInterface {
    
    fn enter(&mut self){
        if self.cursory != self.starty + self.height - 1{
            plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::Black));
            self.move_cursor(self.startx, self.cursory + 1);
        }
    }

    fn initialize(&mut self,  am: char){
        self.cursorx = self.startx;
        self.id = am;
        self.cursory = self.starty;
        self.normal_border();
        self.init = true;
        self.create_files();
        self.display_files();
    }

    fn create_files(&mut self){
        let hello = self.file_sys.open_create("hello").unwrap();
        self.file_sys.write(hello, r#"print("Hello, world!")"#.as_bytes()).unwrap();
        self.file_sys.close(hello).unwrap();

        let nums = self.file_sys.open_create("nums").unwrap();
        self.file_sys.write(nums, r#"print(1)
print(257)
"#.as_bytes()).unwrap();
        self.file_sys.close(nums).unwrap();

        let average = self.file_sys.open_create("average").unwrap();
        self.file_sys.write(average, r#"sum := 0
count := 0
averaging := true
while averaging {
    num := input("Enter a number:")
    if (num == "quit") {
        averaging := false
    } else {
        sum := (sum + num)
        count := (count + 1)
    }
}
print((sum / count))")"#.as_bytes()).unwrap();
        self.file_sys.close(average).unwrap();

        let pi = self.file_sys.open_create("pi").unwrap();
        self.file_sys.write(pi, r#"sum := 0
i := 0
neg := false
terms := input("Num terms:")
while (i < terms) {
    term := (1.0 / ((2.0 * i) + 1.0))
    if neg {
        term := -term
    }
    sum := (sum + term)
    neg := not neg
    i := (i + 1)
}
print((4 * sum))"#.as_bytes()).unwrap();
        self.file_sys.close(pi).unwrap();
        
    }

    fn display_files(&mut self){
        let files = self.file_sys.list_directory().unwrap();
        let mut c = self.startx;
        let mut r = self.starty;
        for f in 0..files.0{
            let mut n = str::from_utf8(&files.1[f]).unwrap().trim_matches(char::from(0));
            if f == 0{
                n = "hello";
            }
            if f == self.active_file{
                plot_str("            ", c, r, ColorCode::new(Color::Black, Color::Cyan));
                plot_str(n, c, r, ColorCode::new(Color::Black, Color::Cyan));
            }
            else {
                plot_str("            ", c, r, ColorCode::new(Color::Black, Color::Black));
                plot_str(n, c, r, ColorCode::new(Color::Cyan, Color::Black));
            }
            if (f + 1) % 3 == 0{
                r += 1;
                c = self.startx;
            }
            else{
                c += 12;
            }
        }
    }

    fn change_active_file(&mut self, pos: bool){
        let num = self.file_sys.list_directory().unwrap().0;
        if pos{
            if self.active_file < num - 1{
                self.active_file += 1;
            }
            else{
                self.active_file = 0;
            }
        } else{
            if self.active_file != 0{
                self.active_file -= 1;
            } else{
                self.active_file = num - 1;
            }
        }
    }

    fn active_border(&mut self){
        for x in self.startx - 1..self.startx + self.width{
            if self.starty != 2 || (x != self.startx + 17 && x != self.startx + 18) {
                plot('.', x, self.starty - 1, ColorCode::new(Color::White, Color::Magenta));
                plot('.', x, self.starty + self.height, ColorCode::new(Color::White, Color::Magenta));
            }
        }
        for y in self.starty - 1..self.starty + self.height{
            plot('.', self.startx - 1, y, ColorCode::new(Color::White, Color::Magenta));
            plot('.', self.startx + self.width, y, ColorCode::new(Color::White, Color::Magenta));
        }
        plot('F', self.startx + 17, self.starty - 1, ColorCode::new(Color::Cyan, Color::Black));
        plot(self.id, self.startx + 18 , self.starty - 1, ColorCode::new(Color::Cyan, Color::Black));
    }
    fn normal_border(&mut self){
        for x in self.startx - 1..self.startx + self.width{
            if self.starty != 2 || (x != self.startx + 17 && x != self.startx + 18) {
                plot('.', x, self.starty - 1, ColorCode::new(Color::White, Color::Black));
                plot('.', x, self.starty + self.height, ColorCode::new(Color::White, Color::Black));    
            }
        }
        for y in self.starty - 1..self.starty + self.height{
            plot('.', self.startx - 1, y, ColorCode::new(Color::White, Color::Black));
            plot('.', self.startx + self.width, y, ColorCode::new(Color::White, Color::Black));
        }
        plot('F', self.startx + 17, self.starty - 1, ColorCode::new(Color::Cyan, Color::Black));
        plot(self.id, self.startx + 18 , self.starty - 1, ColorCode::new(Color::Cyan, Color::Black));
    }

    fn show_cursor(&mut self){
        plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::White));
    }

    fn add_letter(&mut self, key: char){
        //self.text[self.cursory] = self.text[self.cursory] + key;
        if self.cursory < self.starty + self.height && self.cursorx < self.startx + self.width{
            plot(key, self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));

                    if self.cursorx < self.startx + self.width - 1{
                        self.cursorx += 1;
                    }
                    else{
                        if self.cursory != self.starty + self.height - 1{
                            self.move_cursor(self.startx, self.cursory + 1); 
                            self.show_cursor();   
                        }
                        
                        
                    }
                    
        }
        
    }

    fn move_cursor(&mut self, x: usize, y:usize){
        self.cursorx = x;
        self.cursory = y;
        self.show_cursor();
    }

    

   
}
