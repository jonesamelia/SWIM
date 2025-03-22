#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SwimInterface<'a> {
    text: [&'a str; BUFFER_HEIGHT],
    num_letters: usize,
    next_letter: usize,
    cursorx: usize,
    cursory: usize,
    height: usize,
    width: usize,
    init: bool,
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

impl Default for SwimInterface<'_> {
    fn default() -> Self {
        Self {
            text: [" "; BUFFER_HEIGHT],
            num_letters: 1,
            next_letter: 1,
            cursorx: 0,
            cursory: 0,
            width: BUFFER_WIDTH,
            height: BUFFER_HEIGHT,
            init: false,
        }
    }
}

impl SwimInterface<'_> {
    

    pub fn tick(&mut self) {
        self.initialize();
    }

    fn initialize(&mut self){
        self.show_cursor();
    }
    fn show_cursor(&mut self){
        plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::White));
    }

    fn add_letter(&mut self, key: char){
        //self.text[self.cursory] = self.text[self.cursory] + key;
        if self.cursory != self.height -1 || self.cursorx != self.width-1{
            plot(key, self.cursorx, self.cursory, ColorCode::new(Color::Cyan, Color::Black));

                    if self.cursorx < self.width - 1{
                        self.cursorx += 1;
                    }
                    else{
                        self.move_cursor(0, self.cursory + 1);
                    }
                    self.show_cursor();
        }
        
    }

    fn move_cursor(&mut self, x: usize, y:usize){
        self.cursorx = x;
        self.cursory = y;
        self.show_cursor();
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        match key{
            '\n' =>{
                if self.cursory != self.height-1{
                    plot(' ', self.cursorx, self.cursory, ColorCode::new(Color::Black, Color::Black));
                    self.move_cursor(0, self.cursory +1);
                }
                
            }
            _ =>{
                if is_drawable(key) {
                    self.add_letter(key);
                }    
            }
        }
        
    }
}
