/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: cga                                                             ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: This module provides functions for doing output on the CGA text ║
   ║         screen. It also supports a text cursor position stored in the   ║
   ║         hardware using ports.                                           ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Michael Schoetter, Univ. Duesseldorf, 6.2.2024                  ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use spin::Mutex;
use crate::kernel::cpu as cpu;

/// Global CGA instance, used for screen output in the whole kernel.
/// Usage: let mut cga = cga::CGA.lock();
///        cga.print_byte(b'X');
pub static CGA: Mutex<CGA> = Mutex::new(CGA::new());

/// All 16 CGA colors.
#[repr(u8)] // store each enum variant as an u8
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Pink       = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    LightPink  = 13,
    Yellow     = 14,
    White      = 15,
}

pub const CGA_STD_ATTR: u8 = (Color::Black as u8) << 4 | (Color::Green as u8);

const CGA_BASE_ADDR: *mut u8 = 0xb8000 as *mut u8;
const CGA_ROWS: usize = 25;
const CGA_COLUMNS: usize = 80;

const CGA_INDEX_PORT: u16 = 0x3d4; // select register
const CGA_DATA_PORT: u16 = 0x3d5;  // read/write register
const CGA_HIGH_BYTE_CMD: u8 = 14;  // cursor high byte
const CGA_LOW_BYTE_CMD: u8 = 15;   // cursor high byte

pub struct CGA {
    index_port: cpu::IoPort,
    data_port: cpu::IoPort
}

impl CGA {
    /// Create a new CGA instance.
    const fn new() -> CGA {
        CGA {
            index_port: cpu::IoPort::new(CGA_INDEX_PORT),
            data_port: cpu::IoPort::new(CGA_DATA_PORT)
        }
    }

    /// Clear CGA screen and set cursor position to (0, 0).
    pub fn clear(&mut self) {
        for x in 0..CGA_COLUMNS
        {
            for y in 0..CGA_ROWS
            {
                self.print_byte_at_nowrapping(' ' as u8, x, y);
            }
        }
        self.setpos(0,0);
    }

    /// Display the `character` at the given position `x`,`y` with attribute `attrib`.
    pub fn show(&mut self, x: usize, y: usize, character: char, attrib: u8) {
        if x > CGA_COLUMNS || y > CGA_ROWS {
            return;
        }

        let pos = (y * CGA_COLUMNS + x) * 2;

        // Write character and attribute to the screen buffer.
        //
        // Unsafe because we are writing directly to memory using a pointer.
        // We ensure that the pointer is valid by using CGA_BASE_ADDR
        // and checking the bounds of x and y.
        unsafe {
            CGA_BASE_ADDR.offset(pos as isize).write(character as u8);
            CGA_BASE_ADDR.offset((pos + 1) as isize).write(attrib);
        }
    }

    /// Return cursor position `x`,`y`
    pub fn getpos(&mut self) -> (usize, usize) {

        // Get cursor position (Lower Byte)
        let low_csr :u8;
        unsafe {
            self.index_port.outb(CGA_LOW_BYTE_CMD); // Put "lower cursor position"- index in index-register
            low_csr = self.data_port.inb(); // Get data at position stated inside index-register
        }

        // Get cursor position (Higher Byte)
        let high_csr :u8;
        unsafe {
            self.index_port.outb(CGA_HIGH_BYTE_CMD);
            high_csr = self.data_port.inb();
        }

        // Combine Bytes
        let pos_crs :u16 = ((high_csr as u16) << 8) + (low_csr as u16);

        // position -> X,Y
        let x :usize = (pos_crs % CGA_COLUMNS as u16) as usize;
        let y :usize = (pos_crs / CGA_COLUMNS as u16) as usize;

        (x, y)
    }

    /// Set cursor position `x`,`y` 
    pub fn setpos(&mut self, x: usize, y: usize) {

        // X,Y -> position
        let pos_crs :u16 = (y * CGA_COLUMNS + x) as u16;

        // Set cursor position (Lower Byte)
        let low_csr :u8 = pos_crs as u8; // Nur untere 8 Bits
        unsafe {
            self.index_port.outb(CGA_LOW_BYTE_CMD); // Put "lower cursor position"- index in index-register
            self.data_port.outb(low_csr); // Set data at position stated inside index-register
        }

        // Set cursor position (Higher Byte)
        let high_csr :u8 = (pos_crs >> 8) as u8; // Nur Obere 8 Bits
        unsafe {
            self.index_port.outb(CGA_HIGH_BYTE_CMD);
            self.data_port.outb(high_csr);
        }
    }

    /// Print byte `b` at actual position cursor position `x`,`y`
    pub fn print_byte(&mut self, b: u8) {
        let (mut x , mut y) = self.getpos();

        // Check for new line
        if b == '\n' as u8
        {
            x = 0;
            y = y+1;
            self.setpos(x, y);
            return
        }

        self.print_byte_at_nowrapping(b, x, y);

        // TODO: Fix Scrolling
        // Set new Position
        if y >= CGA_ROWS // Scroll Up
        {
            self.scrollup();
            self.setpos(0, CGA_ROWS-1);
        }
        else if x+1 > CGA_COLUMNS // Linebrake
        {
            self.setpos(0, y+1);
        }
        else 
        {
            self.setpos(x+1, y);
        }
    }

    /// Print byte `b` at `x`,`y`
    pub fn print_byte_at_nowrapping(&mut self, b: u8, x: usize, y: usize) {
        // calculate position in storage
        let pos :usize = CGA_BASE_ADDR as usize + 2 * ((x+y*CGA_COLUMNS) as usize);

        // Generate output
        let output_data :u16 = b as u16 | ((CGA_STD_ATTR as u16) << 8);

        unsafe{
            *(pos as *mut u16) = output_data;
        }
    }

    /// Scroll text lines by one to the top.
    pub fn scrollup(&mut self) {
        for y in 1..CGA_ROWS
        {
            for x in 0..CGA_COLUMNS
            {
                let symbol: u8 = self.get_byte(x, y);
                self.print_byte_at_nowrapping(symbol, x, y-1);
            }
        }
    }

    fn get_byte(&mut self, x: usize, y: usize) -> u8 {
        // Calculate position
        let pos :usize = CGA_BASE_ADDR as usize + 2 * ((x+y*CGA_COLUMNS) as usize);
        let pos_ptr: *mut u8 = pos as *mut u8;

        // Get Value
        let byte: u8;
        unsafe {
            byte = pos_ptr.read_volatile();
        }

        byte
    }

    /// Helper function returning an attribute byte for the given parameters `bg`, `fg`, and `blink`
    /// Note: Blinking characters do not work in QEMU, but work on real hardware.
    ///       Support for blinking characters is optional and can be removed, if you want.
    pub fn attribute(&mut self, bg: Color, fg: Color, blink: bool) -> u8 {

        let fg_att :u8 = fg as u8; // Forground-Color
        let bg_att :u8 = (bg as u8) << 3; // Background-Color
        let blink_att :u8 = (blink as u8) << 7; // Blinking

        let result :u8 = fg_att | bg_att | blink_att;

        result
    }
}