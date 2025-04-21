use alloc::boxed::Box;

pub fn run () {

    struct Test {
        a: u32,
        b: u32,
    }
    let mut test = Test { a: 1, b: 2 };

    Box::new(test);

}
