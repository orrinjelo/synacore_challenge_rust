use log::{trace, debug, info, warn, error};
use crate::vm::Vm;
use crate::util::{get_file_as_byte_vec};
use regex::Regex;
use std::{error::Error};
use termion::{event::Key};
use crate::util::event::{Event, Events};
use crossbeam;
use std::io;
use termion::{raw::IntoRawMode};

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
    messages: Vec<String>,
    events: Events,
}

impl Console
{
    pub fn new(input_file: String, memsize: usize) -> Console {
        let _stdout = io::stdout().into_raw_mode().unwrap();
        Console {
            vm: Vm::new(
                get_file_as_byte_vec(&input_file),
                memsize,
            ),
            running: true,
            input: String::new(),
            messages: Vec::new(),
            events: Events::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while self.running {
            self.handle_input()?;
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn Error>>  {
        // Handle input
        if let Event::Input(input) = self.events.next()? {
            match input {
                Key::Char('\n') => {
                    if !self.maybe_parse_input() {
                        self.messages.push(self.input.drain(..).collect());
                    }
                    // print!("> ");
                },
                Key::Char(c) => {
                    println!("Pushing {}", c);
                    self.input.push(c);
                },
                Key::Backspace => {
                    self.input.pop();
                },
                Key::Ctrl('a') => {
                    println!("Pausing...");
                    self.vm.pause();
                },
                Key::Ctrl(c) => {
                    println!("Caught a CTRL-{}", c);
                },
                Key::Esc => {
                    // app.input_mode = InputMode::Normal;
                    // events.enable_exit_key();
                    println!("ESC!");
                },
                _ => {
                    // println!("test.");
                },
            }
        }
        Ok(())
    }

    fn maybe_parse_input(&mut self) -> bool {
        if self.input == "!run".to_string() {
            crossbeam::scope(|scope| {
                scope.spawn(move || {
                    self.vm.execute_until_done();
                });
            });
            return true;
        } else if self.input == "!reset".to_string() {
            self.reset_vm();
            return true;
        } else if self.input.contains("!break") {
            self.add_breakpoint();
            return true;
        } else if self.input == "!quit".to_string() {
            self.running = false;
            return true;
        } else {
            self.vm.insert_buffer(self.input.clone());
        }
        return false;
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
    }
}