
pub extern "C" fn sys_print(ptr: *const u8, len: u64) {
    // Get the slice containing the string
    let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    let s = core::str::from_utf8(slice).unwrap_or("<Invalid UTF-8>");

    print!("{}",s);
}
