const CGA_BUFFER: *mut u8 = 0x3000_0000_0000 as *mut u8;

pub fn write_char(x: usize, y: usize, c: u8, color: u8) {
    let index = (y * 80 + x) * 2;

    unsafe {
        *CGA_BUFFER.add(index) = c;
        *CGA_BUFFER.add(index + 1) = color;
    }
}