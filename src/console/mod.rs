use log::{trace, debug, info, warn, error};
use crate::vm::Vm;
use crate::util::{get_file_as_byte_vec};
use regex::Regex;
use std::{error::Error};
// use crossbeam;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::cursor::DetectCursorPos;
use termion::{color};
use std::io::{Write, stdout, stdin};
use std::cmp;
use async_std::task;
// use ctrlc;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use crate::util::event::{Event, Events};

#[allow(dead_code)]
fn _get_rid_of_log_unused_import_warnings() {
    trace!("Example trace.");
    debug!("Example debug.");
    info!("Example info.");
    warn!("Example warn.");
    error!("Example error.");
}

#[allow(dead_code)]
pub struct Console {
    vm: Vm,
    running: bool,
    input: String,
    output: String,
    history: Vec<String>,
    vm_input: String,
    vm_output: String,
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
            output: String::new(),
            history: Vec::new(),
            vm_input: String::new(),
            vm_output: String::new(),
            // events: Events::new(),
        }
    }

    pub fn cprint(&mut self, message: &str) {
        self.output = message.to_string();
    }

    #[allow(dead_code)]
    pub fn vm_print(&mut self, message: &str) {
        self.vm_output = message.to_string();
    }

    #[allow(dead_code)]
    pub fn vm_get_input(&mut self) -> String {
        self.vm_input.clone()
    }


    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout,
               "{}{}{}> ",
               termion::cursor::Goto(1, 1),
               termion::clear::All,
               color::Fg(color::White)
        ).unwrap();
        stdout.flush().unwrap();

        for input in stdin.keys() {
            match input.unwrap() {
                Key::Char('\n') => {
                    let s = self.clone()
                    if !s.maybe_parse_input().await {
                        self.history.push(self.input.drain(..).collect());
                    }
                    if self.history.len() > 0 {
                        for entry in 0..cmp::min(6, self.history.len()) {
                            write!(stdout,
                               "{}{}{}{}{}",
                               termion::cursor::Goto(1, 3+entry as u16),
                               termion::clear::CurrentLine,
                               color::Fg(color::Yellow),
                               self.history[(self.history.len() as i16 - entry as i16 - 1) as usize],
                               color::Fg(color::Reset)
                            ).unwrap();
                        }
                    }

                    write!(stdout,
                       "{}{}{}> ",
                       termion::cursor::Goto(1, 1),
                       termion::clear::CurrentLine,
                       color::Fg(color::White)
                    ).unwrap();

                    self.cprint("");
                },
                Key::Char(c) => {
                    print!("{}", c);
                    self.input.push(c);
                },
                Key::Backspace => {
                    let (x1,y1) = stdout.cursor_pos()?;
                    self.input.pop();
                    write!(stdout,
                       "{}{}",
                       termion::cursor::Goto(x1-1,y1),
                       color::Fg(color::White)
                    ).unwrap();
                },
                Key::Ctrl('a') => {
                    self.cprint("Pausing...");
                    self.vm.pause();
                },
                Key::Ctrl('c') => {
                    self.cprint("Time to quit.");
                    break;
                },
                Key::Ctrl(c) => {
                    self.cprint(&format!("Caught a CTRL-{}", c));
                },
                Key::Esc => {
                    self.cprint("ESC!");
                    break;
                },
                _ => {
                    // self.cprint("");
                },
            }

            let (x,y) = stdout.cursor_pos()?;

            write!(stdout,
               "{}{}{}== {}",
               termion::cursor::Goto(1, 2),
               termion::clear::CurrentLine,
               color::Fg(color::Cyan),
               self.output
            ).unwrap();

            write!(stdout,
               "{}{}",
               termion::cursor::Goto(x,y),
               color::Fg(color::White)
            ).unwrap();

            stdout.flush().unwrap();
        }
        write!(stdout, "{}", termion::cursor::Show).unwrap();
        Ok(())
    }

    async fn maybe_parse_input(&mut self) -> bool {
        if self.input == "!run".to_string() {
            task::spawn(self.vm.execute_until_done()).await;
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

    // fn handle_input(&mut self) -> Result<(), Box<dyn Error>>  {
    //     // Handle input
    //     std::io::stdout().flush();
    //     let bytes = std::io::stdin().read_line(&mut self.input)?;

    //     trace!("{} bytes read", bytes);
    //     trace!("Input: {}", self.input);

    //     if self.input.trim().contains("!run") {
    //         crossbeam::scope(|scope| {
    //             scope.defer(move || {
    //                 self.vm.execute_until_done();
    //             });
    //         });
    //         trace!("Thread spawned.");
    //     } else if self.input.trim().eq("!reset") {
    //         self.reset_vm();
    //     } else if self.input.contains("!break") {
    //         self.add_breakpoint();
    //     } else if self.input.trim().eq("!quit") {
    //         self.reset_vm();
    //         self.running = false;
    //     } else {
    //         self.vm.insert_buffer(self.input.clone());
    //     }
    //     Ok(())
    // }

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