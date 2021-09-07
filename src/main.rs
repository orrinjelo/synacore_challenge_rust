mod vm;
mod console;
mod util;

use log::{Level}; // trace, debug, info, warn, error
use env_logger;
use rustop::opts;
use std::{error::Error};
use futures::executor::block_on;
use std::io::Write as IoWrite;


fn main() -> Result<(), Box<dyn Error>> {

    let opts = opts! {
        synopsis "Synacor Challenge 2020";
        opt memsize:usize=32768, desc: "Memory size of VM";
        opt input_file:String="challenge.bin".to_string(), desc: "Input file";
        opt disassemble:Option<String>, desc: "Dissassemble input into ASM.";
        opt bp:Option<usize>, desc: "Add a breakpoint.";
    };

    let (args, _rest) = opts.parse_or_exit();

    // Set up logging
    let mut c = console::Console::new(args.input_file, args.memsize);

    env_logger::builder()
        .format(|buf, record| {
            let mut style = buf.style();

            let color = match record.level() {
                Level::Trace => env_logger::fmt::Color::Magenta,
                Level::Debug => env_logger::fmt::Color::Cyan,
                Level::Info  => env_logger::fmt::Color::Green,
                Level::Warn  => env_logger::fmt::Color::Yellow,
                Level::Error => env_logger::fmt::Color::Red,
            };

            style.set_color(color);
           writeln!(buf, "{}: {}", style.value(record.level()), record.args())
        })
        .init();

    block_on(c.run())?;

    Ok(())
}
