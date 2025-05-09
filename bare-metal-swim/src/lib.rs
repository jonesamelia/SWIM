#![no_std]

use file_system_template::FileSystem;
use gc_heap_template::GenerationalHeap;
use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, plot_str, plot_num, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};
use ramdisk::RamDisk;
use simple_interp::{Interpreter, InterpreterOutput, ArrayString};

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive, str,
};

#[derive(PartialEq)]
enum CurrentSwim {
    Running,
    DisplayingFiles,
    EditingFiles,
    AwaitingInput,
    DisplayingOutput
}

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
const MAX_TOKENS: usize = 100; 
const MAX_LITERAL_CHARS: usize = 15;
const STACK_DEPTH: usize = 20; 
const MAX_LOCAL_VARS: usize = 10; 
const HEAP_SIZE: usize = 256; 
const MAX_HEAP_BLOCKS: usize = HEAP_SIZE;


pub struct SwimInterface {
    text: [[char; 40]; 35],
    outputy: usize,
    inputy: usize,
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
    running_file: bool,
    inpstring: ArrayString<WIN_WIDTH>,
    status: CurrentSwim,
    interpreter: Option<Interpreter<MAX_TOKENS, MAX_LITERAL_CHARS, STACK_DEPTH, MAX_LOCAL_VARS, WIN_WIDTH, GenerationalHeap<HEAP_SIZE, MAX_HEAP_BLOCKS, 2>>>,
    active_file: usize,
    topi: usize
}


pub struct Swim{
    windows: [SwimInterface; 4],
    window_ticks: [usize; 4],
    init: bool,
    active_win: usize,
    displaying_files: bool,
    creating_file: bool,
    nextt: usize
}

impl Default for Swim{
    fn default() -> Self{
        Self{
            windows: [SwimInterface::default(), SwimInterface::default(), SwimInterface::default(), SwimInterface::default()],
            window_ticks: [0; 4],
            init: false,
            active_win: 0,
            displaying_files: true,
            creating_file: false,
            nextt: 0
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
            text: [[' '; 40]; 35],
            outputy: 2,
            inputy: 2,
            interpreter: None,
            num_letters: 1,
            next_letter: 1,
            cursorx: 1,
            cursory: 2,
            startx: 1,
            starty: 2,
            width: 34,
            height: 10,
            id: ' ',
            init: false,
            active: false,
            inpstring: ArrayString::default(),
            file_sys: FileSystem::new(RamDisk::new()),
            displaying_files: true,
            status: CurrentSwim::DisplayingFiles,
            running_file: false,
            active_file: 0,
            topi: 0

        }
    }
}

impl Swim{
    pub fn tick(&mut self){
        if self.init == false{
            self.initialize();
        }
        self.windows[self.active_win].tick();
        if self.creating_file{
            plot_str("Filename: ", 0, 0, ColorCode::new(Color::Cyan, Color:: Black));
        }
        
        let mut programs_running: [usize; 4] = [0; 4];
        let mut running_count = 0;
        for i in 0..self.windows.len(){
            if self.windows[i].status == CurrentSwim::Running{
    
             running_count += 1;    
            
            }
        }
        if running_count > 0{
            let next_w = programs_running[self.nextt % running_count];
            self.window_ticks[next_w] += 1;

            self.windows[next_w].tick();
            self.nextt = (self.nextt + 1) % running_count;
        }
        self.plot_ins_counts();


        //else{
         //   if !self.windows[self.active_win].running_file{
          //      self.windows[self.active_win].show_cursor();  
          //  }       
        //}
    }

    fn plot_ins_counts(&mut self){
        plot_str("F1", 72, 0, ColorCode::new(Color::Cyan, Color::Black));
        plot_num(self.window_ticks[0] as isize, 72, 1, ColorCode::new(Color::Cyan, Color::Black));
        plot_str("F2", 72, 2, ColorCode::new(Color::Cyan, Color::Black));
        plot_num(self.window_ticks[1] as isize, 72, 3, ColorCode::new(Color::Cyan, Color::Black));
        plot_str("F3", 72, 4, ColorCode::new(Color::Cyan, Color::Black));
        plot_num(self.window_ticks[2] as isize, 72, 5, ColorCode::new(Color::Cyan, Color::Black));
        plot_str("F4", 72, 6, ColorCode::new(Color::Cyan, Color::Black));
        plot_num(self.window_ticks[3] as isize, 72, 7, ColorCode::new(Color::Cyan, Color::Black));
    }

    fn initialize(&mut self){
        plot_str("Header", 0, 0, ColorCode::new(Color::Cyan, Color::Black));
        self.windows[0].active = true;
        self.windows[0].initialize('1');
        self.windows[1].startx = 36;
        self.windows[1].starty = 2;
        self.windows[1].outputy = 2;
        self.windows[1].initialize('2');
        self.windows[2].starty = 13;
        self.windows[2].starty = 13;
        self.windows[2].inputy = 13;
        self.windows[2].initialize('3');
        self.windows[3].startx = 36;
        self.windows[3].starty = 13;
        self.windows[3].inputy = 13;
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
            KeyCode::F5 => {
                
            }
            KeyCode::F6 => {
                if self.windows[self.active_win].status == CurrentSwim::Running{
                    self.windows[self.active_win].status = CurrentSwim::DisplayingFiles;
                    self.windows[self.active_win].display_files();
                } 
                else if self.windows[self.active_win].status == CurrentSwim::DisplayingOutput{
                    self.windows[self.active_win].status = CurrentSwim::DisplayingFiles;
                    self.windows[self.active_win].display_files();
                }
                else if self.windows[self.active_win].status == CurrentSwim::AwaitingInput{
                    self.windows[self.active_win].status = CurrentSwim::DisplayingFiles;
                    self.windows[self.active_win].display_files();
                }

            }
            KeyCode::ArrowLeft => {
                if self.displaying_files{
                    self.windows[self.active_win].change_active_file(false);    
                } 
                else{
                    self.windows[self.active_win].sidescroll(false);
                }
            }
            KeyCode::ArrowRight => {
                if self.displaying_files{
                    self.windows[self.active_win].change_active_file(true);    
                }  
                else{
                    self.windows[self.active_win].sidescroll(true);
                }
            }
            KeyCode::ArrowUp => {
                if self.displaying_files{
                    self.windows[self.active_win].change_active_file(true);    
                }  
                else{
                    self.windows[self.active_win].verticalscroll(false);
                }
            }
            KeyCode::ArrowDown => {
                if self.displaying_files{
                    self.windows[self.active_win].change_active_file(true);    
                }  
                else{
                    self.windows[self.active_win].verticalscroll(true);
                }
            }
            KeyCode::Backspace => {
                if !self.displaying_files{
                    self.windows[self.active_win].backspace();
                }
            }
            KeyCode::Delete => {
                if !self.displaying_files{
                    self.windows[self.active_win].backspace();
                }
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
                if self.windows[self.active_win].status == CurrentSwim::AwaitingInput{
                    self.windows[self.active_win].enterinput();
                }
            }
            '\u{08}' | '\u{7f}' => {
                if !self.displaying_files{
                    self.windows[self.active_win].backspace();
                }
            }
            'r' =>{
                if self.windows[self.active_win].status == CurrentSwim::DisplayingFiles{
                    self.windows[self.active_win].run_active_file();    
                }  
            }
            _ =>{
                if is_drawable(key) {
                    if !self.displaying_files{
                        self.windows[self.active_win].add_letter(key);    
                    }
                    if self.windows[self.active_win].status == CurrentSwim::AwaitingInput{
                        self.windows[self.active_win].add_letter(key);
                    }
                    
                }    
            }
        }
    }
}

impl InterpreterOutput for SwimInterface {
    fn print(&mut self, chars: &[u8]) {
       let output = match core::str::from_utf8(chars) {
        Ok(s) => s,
        Err(_) => return,
        };
       let row = self.starty;
       let col = self.startx;
       let bla = ColorCode::new(Color::Black, Color::Black);
       self.clear_screen();
       plot_str(output, col, row, ColorCode::new(Color::Cyan, Color::Black));
       self.outputy += 1;
       self.inputy = row + 1;
       
    }
}

impl SwimInterface {
    
    fn enter(&mut self){
        if self.cursory != self.starty + self.height - 1{
            plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::Black));
            self.move_cursor(self.startx, self.cursory + 1);
        }else{
            self.topi += 1;
            self.printscreen();
        }
    }

    fn enterinput(&mut self){
        if self.status == CurrentSwim::AwaitingInput{
            let mut instring = ArrayString::default();
            for i in 0..self.num_letters-1{
                instring.push_char(self.text[i][self.inputy - self.starty + self.topi]);
            } 
            self.cursorx = self.startx;
            self.cursory = self.starty;
            self.num_letters = 0;
            self.status = CurrentSwim::Running;
            self.running_file = true;
            self.inpstring = instring;
        }
    }

    fn initialize(&mut self,  am: char){
        self.cursorx = self.startx;
        self.id = am;
        self.cursory = self.starty;
        self.normal_border();
        self.init = true;
        if self.displaying_files{
            self.create_files();
            self.display_files();    
        }
        
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
            if f == self.active_file{
                plot_str("           ", c, r, ColorCode::new(Color::Black, Color::Cyan));
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
                c += 11;
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

    fn tick(&mut self){
        if self.status == CurrentSwim::Running{
                
            if let Some(mut inpt) = self.interpreter.take(){
                let input_s = match self.inpstring.as_str(){
                    Ok(s) => s,
                    Err(_) => "",
                };
                if !input_s.is_empty(){
                    inpt.provide_input(input_s).unwrap();
                    self.inpstring.clear();
                }
                let res = inpt.tick(self);

                match res{
                    simple_interp::TickStatus::Continuing => {},
                    simple_interp::TickStatus::Finished => {
                        self.status = CurrentSwim::DisplayingOutput;
                        self.running_file = false;

                    },
                    simple_interp::TickStatus::AwaitInput => {
                        self.status = CurrentSwim::AwaitingInput;
                        //self.clear_line(self.startx + 2);
                        self.num_letters = 0;
                        self.cursory = self.starty + 1;
                        self.next_letter = 0;

                    }  
                }
                self.interpreter = Some(inpt);
            }
                
        }
        if self.status == CurrentSwim::AwaitingInput{
            self.outputy = 0;
        }
        if self.status == CurrentSwim::DisplayingFiles{
            self.display_files();
        }
    }

    fn run_active_file(&mut self){
       let files = self.file_sys.list_directory().unwrap().1;
       let f_name = &str::from_utf8(&files[self.active_file]).unwrap().trim_matches(char::from(0));
       let fd = self.file_sys.open_read(f_name.trim()).unwrap();
       let mut buffer = [0; MAX_FILE_BYTES];
       self.file_sys.read(fd, &mut buffer).unwrap();
       let file = str::from_utf8(&buffer).unwrap().trim_matches(char::from(0));
       self.file_sys.close(fd).unwrap();
       self.status = CurrentSwim::Running;
       self.outputy = 0;
       self.cursory = 0;
       self.cursorx = 0;
       self.num_letters = 0;
       self.next_letter = 0;
       self.running_file = true;
       self.cursorx = self.startx;
       self.interpreter = Some(Interpreter::new(file));
       
    }

    fn clear_line(&self, row: usize){
        for x in self.startx..self.startx + self.width{
            plot(' ', x, self.starty - 1, ColorCode::new(Color::Black, Color::Black));
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
        let mut row = self.cursorx;
        //self.text[self.cursory] = self.text[self.cursory] + key;
        if self.status == CurrentSwim::AwaitingInput{
            self.cursory = self.inputy;
            self.num_letters += 1;
        }
        if self.cursory < self.starty + self.height && self.cursorx < self.startx + self.width{
            plot(key, self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));
            self.text[self.cursorx - self.startx][self.cursory - self.starty + self.topi] = key;
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
        self.num_letters += 1;
        
    }

    fn backspace(&mut self){
        if self.cursorx == self.startx && self.cursory != self.starty{
            plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::Black));
            self.move_cursor(self.startx + self.width - 1, self.cursory -1);
        }
        else if self.cursorx != self.startx {
            plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::Black));
            self.move_cursor(self.cursorx - 1,self.cursory);
            self.show_cursor();
            self.text[self.cursorx - self.startx][self.cursory - self.starty + self.topi] = ' ';
        }
    }
    fn sidescroll(&mut self, pos: bool){
        let mut newx = self.cursorx;
        if pos{ 
            if self.cursorx != self.startx + self.width - 1{
                newx += 1;
            }
            else{newx = self.startx;}
        }
        else{ 
            if self.cursorx != self.startx{ 
                newx -= 1;    
            }
            else{
                newx = self.startx + self.width - 1;
            }
        }   
        plot(self.text[self.cursorx - self.startx][self.cursory - self.starty + self.topi], self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));
        self.move_cursor(newx, self.cursory);
    }
    fn verticalscroll(&mut self, pos:bool){
        let mut newy = self.cursory;
        if pos{ 
            if self.cursory != self.starty + self.height - 1{
                newy += 1;
                plot(self.text[self.cursorx - self.startx][self.cursory - self.starty + self.topi], self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));
                self.move_cursor(self.cursorx, newy);
            }
            else if self.topi != 40 - self.height{
                self.topi += 1;
                self.printscreen();
            }
        }
        else{ 
            if self.cursory != self.starty{ 
                newy -= 1; 
                plot(self.text[self.cursorx - self.startx][self.cursory - self.starty + self.topi], self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));
                self.move_cursor(self.cursorx, newy);   
            }
            else if self.topi != 0{
                self.topi -= 1;
                self.printscreen();
            }
        }   
    }
    fn printscreen(&mut self){
        for x in self.startx..self.startx + self.width - 1{
            for y in self.starty..self.starty + self.height{
                plot(self.text[x - self.startx][y - self.starty + self.topi], x, y, ColorCode::new(Color::Cyan, Color::Black));
            }
        }
    }
    fn clear_screen(&mut self){
        for x in self.startx..self.startx + self.width - 1{
            for y in self.starty..self.starty + self.height{
                plot(' ', x, y, ColorCode::new(Color::Black, Color::Black));
            }
        }
    }
    fn move_cursor(&mut self, x: usize, y:usize){
        self.cursorx = x;
        self.cursory = y;
        self.show_cursor();
    }

    

   
}
