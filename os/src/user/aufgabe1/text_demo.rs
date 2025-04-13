use crate::devices::cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println!


pub fn run () {
    // Test Clear
    {
        let mut cga = cga::CGA.lock();
        cga.clear();
    } // Curly brackets for unlock

    // Test scroll
    println!("You can\'t see this!");
    for i in 0..25
    {
        println!("|");
    }
    
    // Test prints
    print!("Hello ");
    println!("World!");

    // Test Escaping
    println!("\n       _~^~^~_\n   \\) /  o o  \\ (/\n     \'_   v   _\'\n     / \'-----\' \\\n");

    // Test Wrapping
    // Should look like: ###### \n # 
    println!("#################################################################################");

    
}
