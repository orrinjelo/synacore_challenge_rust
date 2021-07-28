use log::{trace, debug, info, warn, error};
use crate::vm::Vm;
use crate::util::{get_file_as_byte_vec};
use regex::Regex;
use std::{error::Error, thread};
use crossbeam;
// use ctrlc;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use termion::{event::Key};
// use crate::util::event::{Event, Events};
// use std::io;
// use termion::{raw::IntoRawMode};

#[allow(dead_code)]
fn _get_rid_of_log_unused_import_warnings() {
    trace!("Example trace.");
    debug!("Example debug.");
    info!("Example info.");
    warn!("Example warn.");
    error!("Example error.");
}

// std::io::stdin().read_line(&mut self.buffer).unwrap();
// synopsis "Synacor Challenge 2020";
// opt memsize:usize=32768, desc: "Memory size of VM";
// opt input_file:String="../challenge.bin".to_string(), desc: "Input file";
// opt disassemble:Option<String>, desc: "Dissassemble input into ASM.";
// opt bp:Option<usize>, desc: "Add a breakpoint.";

#[allow(dead_code)]
pub struct Console {
    vm: Vm,
    running: bool,
    input: String,
    history: Vec<String>,
    // events: Events,
}

impl Console
{
    pub fn new(input_file: String, memsize: usize) -> Console {
        // let _stdout = io::stdout().into_raw_mode().unwrap();
        Console {
            vm: Vm::new(
                get_file_as_byte_vec(&input_file),
                memsize,
            ),
            running: true,
            input: String::new(),
            history: Vec::new(),
            // events: Events::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Started console!");

        while self.running {
            self.handle_input()?;
        }

        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn Error>>  {
        // Handle input
        std::io::stdout().flush();
        let bytes = std::io::stdin().read_line(&mut self.input)?;

        trace!("{} bytes read", bytes);
        trace!("Input: {}", self.input);

        if self.input.trim().contains("!run") {
            crossbeam::scope(|scope| {
                scope.defer(move || {
                    self.vm.execute_until_done();
                });
            });
            trace!("Thread spawned.");
        } else if self.input.trim().eq("!reset") {
            self.reset_vm();
        } else if self.input.contains("!break") {
            self.add_breakpoint();
        } else if self.input.trim().eq("!quit") {
            self.reset_vm();
            self.running = false;
        } else {
            self.vm.insert_buffer(self.input.clone());
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn run_vm(&mut self) {
        self.vm.execute_until_done();
    }

    #[allow(dead_code)]
    fn reset_vm(&mut self) {
        self.vm.reset();
    }

    fn add_breakpoint(&mut self) {
        let re = Regex::new(r"!break (\d+)").unwrap();
        let cap = re.captures(&self.input).unwrap();

        let bp = cap[1].parse::<usize>().unwrap();

        self.vm.add_breakpoint(bp);

        debug!("Added breakpoint @ {}", bp);
    }
}