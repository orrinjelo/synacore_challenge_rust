use log::{trace, debug, info, warn, error};
use std::io::{stdin, stdout, Read, Write};
use std::fs::File;
use std::io::LineWriter;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[allow(dead_code)]
fn _get_rid_of_log_unused_import_warnings() {
    trace!("Example trace.");
    debug!("Example debug.");
    info!("Example info.");
    warn!("Example warn.");
    error!("Example error.");
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

const MAX_VAL: usize = 32768;

#[derive(PartialEq, Copy, Clone, Debug)]
enum InstructionCode {
    HALT, // 0,  stop execution and terminate the program
    SET,  // 1,  set register <a> to the value of <b>
    PUSH, // 2,  push <a> onto the stack
    POP,  // 3,  remove the top element from the stack and write it into <a>; empty stack = error
    EQ,   // 4,  set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    GT,   // 5,  set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    JMP,  // 6,  jump to <a>
    JT,   // 7,  if <a> is nonzero, jump to <b>
    JF,   // 8,  if <a> is zero, jump to <b>
    ADD,  // 9,  assign into <a> the sum of <b> and <c> (modulo 32768)
    MULT, // 10, store into <a> the product of <b> and <c> (modulo 32768)
    MOD,  // 11, store into <a> the remainder of <b> divided by <c>
    AND,  // 12, stores into <a> the bitwise and of <b> and <c>
    OR,   // 13, stores into <a> the bitwise or of <b> and <c>
    NOT,  // 14, stores 15-bit bitwise inverse of <b> in <a>
    RMEM, // 15, read memory at address <b> and write it to <a>
    WMEM, // 16, write the value from <b> into memory at address <a>
    CALL, // 17, write the address of the next instruction to the stack and jump to <a>
    RET,  // 18, remove the top element from the stack and jump to it; empty stack = halt
    OUT,  // 19, write the character represented by ascii code <a> to the terminal
    IN,   // 20, read a character from the terminal and write its ascii code to 
          //  <a>; it can be assumed that once input starts, it will continue 
          //  until a newline is encountered; this means that you can safely 
          //  read whole lines from the keyboard and trust that they will be fully 
          //  read
    NOOP  // 21,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Instruction {
    operator: InstructionCode,
    operands: (u16, u16, u16)
}

impl Instruction {
    pub fn parse(code: &Vec<u16>, pc: usize) -> Result<Instruction, Instruction> {
        let op: u16 = code[pc];
        match op {
            0 => Ok(Instruction {
                operator: InstructionCode::HALT,
                operands: (0u16, 0u16, 0u16)
            }),
            1 => Ok(Instruction {
                operator: InstructionCode::SET,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            2 => Ok(Instruction {
                operator: InstructionCode::PUSH,
                operands: (
                    code[pc+1],
                    0,
                    0
                )
            }),
            3 => Ok(Instruction {
                operator: InstructionCode::POP,
                operands: (
                    code[pc+1],
                    0,
                    0
                )
            }),
            4 => Ok(Instruction {
                operator: InstructionCode::EQ,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            5 => Ok(Instruction {
                operator: InstructionCode::GT,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            6 => Ok(Instruction {
                operator: InstructionCode::JMP,
                operands: (
                    code[pc+1],
                    0,
                    0
                )
            }),
            7 => Ok(Instruction {
                operator: InstructionCode::JT,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            8 => Ok(Instruction {
                operator: InstructionCode::JF,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            9 => Ok(Instruction {
                operator: InstructionCode::ADD,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            10 => Ok(Instruction {
                operator: InstructionCode::MULT,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            11 => Ok(Instruction {
                operator: InstructionCode::MOD,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            12 => Ok(Instruction {
                operator: InstructionCode::AND,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            13 => Ok(Instruction {
                operator: InstructionCode::OR,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    code[pc+3]
                )
            }),
            14 => Ok(Instruction {
                operator: InstructionCode::NOT,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            15 => Ok(Instruction {
                operator: InstructionCode::RMEM,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            16 => Ok(Instruction {
                operator: InstructionCode::WMEM,
                operands: (
                    code[pc+1],
                    code[pc+2],
                    0
                )
            }),
            17 => Ok(Instruction {
                operator: InstructionCode::CALL,
                operands: (
                    code[pc+1], 
                    0u16, 
                    0u16
                )
            }),
            18 => Ok(Instruction {
                operator: InstructionCode::RET,
                operands: (
                    0u16, 
                    0u16, 
                    0u16
                )
            }),
            19 => Ok(Instruction {
                operator: InstructionCode::OUT,
                operands: (
                    code[pc+1], 
                    0u16, 
                    0u16
                )
            }),
            20 => Ok(Instruction {
                operator: InstructionCode::IN,
                operands: (
                    code[pc+1], 
                    0u16, 
                    0u16
                )
            }),
            21 => Ok(Instruction {
                operator: InstructionCode::NOOP,
                operands: (0u16, 0u16, 0u16)
            }),            
            _x => {
                // warn!("Instruction code: {}", x);
                // warn!("PC:               {}", pc);
                // warn!("Code[PC]:         {}", code[pc]);
                Err(Instruction {
                    operator: InstructionCode::NOOP,
                    operands: (0u16, 0u16, 0u16)
                })
            }

        }
    }
}

#[derive(Debug, Clone)]
pub struct Vm {
    blueprint: Vec<u16>,    // Max 2**15
    memory: Vec<u16>,    // Max 2**15
    registers: Vec<u16>, // 8
    stack: Vec<u16>,     // Resizeable
    pc: usize,
    stopped: Arc<AtomicBool>,
    buffer: String,
    breakpoints: Vec<usize>,
    paused: Arc<AtomicBool>,
    step_delay: u64,
}

impl Vm {
    pub fn new(input: Vec<u8>, memsize: usize) -> Vm {
        let mut vm = Vm {
            blueprint: Vec::new(),
            memory: Vec::new(),
            registers: vec![0; 8],
            stack: Vec::new(),
            pc: 0,
            stopped: Arc::new(AtomicBool::new(false)),
            buffer: String::new(),
            breakpoints: Vec::new(),
            paused: Arc::new(AtomicBool::new(false)),
            step_delay: 1000,
        };
        for i in 0..input.len()/2 {
            let op: u16 = ((input[i*2+1] as u16) << 8) + (input[i*2] as u16);
            vm.blueprint.push(op);
        }
        for _ in input.len()/2..memsize {
            vm.blueprint.push(0u16);
        }
        vm.reset();

        let p = vm.stopped.clone();
        ctrlc::set_handler(move || {
            warn!("SIGINT caught!");
            p.store(true, Ordering::SeqCst);
        }).expect("Error setting up SIGINT handler.");

        vm
    }

    pub fn reset(&mut self) {
        self.memory = self.blueprint.clone();
        self.registers = vec![0; 8];
        self.stack = Vec::new();
        self.pc = 0;
        self.stopped.store(false, Ordering::SeqCst);
        self.buffer = String::new();
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn add_breakpoint(&mut self, bp: usize) {
        self.breakpoints.push(bp);
    }

    pub fn handle_breakpoint(&self) {
        info!("=== BREAKPOINT ===");
        info!("PC        => {:?}", self.pc);
        info!("MEM[PC]   => {:?}", self.memory[self.pc]);
        info!("REGISTERS => {:?}", self.registers);
        info!("STACK     => {:?}", self.stack);
    }

    pub fn execute_once(&mut self) {
        if self.breakpoints.contains(&self.pc) {
            self.handle_breakpoint();
            pause();
        }
        match Instruction::parse(&self.memory, self.pc) {
            Ok(i) => {
                debug!("=> {:?}", self.memory[self.pc]);
                debug!("== {:?} ==", i);
                debug!("== REGISTERS: {:?}", self.registers);
                match i.operator.clone() {
                    InstructionCode::NOOP => {
                        self.pc += 1;
                    },
                    InstructionCode::HALT => {
                        self.stopped.store(true, Ordering::SeqCst);
                    },
                    InstructionCode::OUT => {
                        let a = i.operands.0;
                        if a >= MAX_VAL as u16 {
                            if self.registers[a as usize % MAX_VAL] > 255 {
                                error!("Invalid character to print! {}", self.registers[a as usize % MAX_VAL]);
                                self.report_error(i, true);
                                pause();
                            } else {
                                print!("{}", self.registers[a as usize % MAX_VAL] as u8 as char);
                            }
                        } else {
                            if a > 255 {
                                error!("Invalid character to print! {}", a);
                                self.report_error(i, true);
                                pause();
                            } else {
                                print!("{}", a as u8 as char);
                            }
                        }
                        self.pc += 2;
                    },
                    InstructionCode::IN => {
                        let a = i.operands.0;
                        while self.buffer.len() == 0 {
                            // std::io::stdin().read_line(&mut self.buffer).unwrap();
                            // Other thread will insert into buffer
                        }
                        if a >= MAX_VAL as u16 {                 
                            self.registers[a as usize % MAX_VAL] = self.buffer.remove(0) as u16;
                        } else {
                            self.memory[a as usize % MAX_VAL] = self.buffer.remove(0) as u16;
                        }
                        self.pc += 2;
                    },
                    InstructionCode::JMP => {
                        let a = i.operands.0;
                        if a >= MAX_VAL as u16 {
                            self.pc = self.registers[a as usize % MAX_VAL] as usize;
                        } else {
                            self.pc = a as usize;
                        }
                    },
                    InstructionCode::CALL => {
                        let a = i.operands.0;
                        self.stack.push(self.pc as u16 + 2);
                        if a >= MAX_VAL as u16 {
                            self.pc = self.registers[a as usize % MAX_VAL] as usize;
                        } else {
                            self.pc = a as usize;
                        }
                    },
                    InstructionCode::RET => {
                        self.pc = self.stack.pop().unwrap() as usize;
                    },
                    InstructionCode::JT => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if a >= MAX_VAL as u16 {
                            if self.registers[a as usize % MAX_VAL] != 0 {
                                self.pc = b as usize;
                            } else {
                                self.pc += 3;
                            }
                        }
                        else if a != 0 {
                            self.pc = b as usize;
                        } else {
                            self.pc += 3
                        }
                    },
                    InstructionCode::JF => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if a >= MAX_VAL as u16 {
                            if self.registers[a as usize % MAX_VAL] == 0 {
                                self.pc = b as usize;
                            } else {
                                self.pc += 3;
                            }
                        }
                        else if a == 0 {
                            self.pc = b as usize;
                        } else {
                            self.pc += 3;
                        }
                    },
                    InstructionCode::SET => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if b >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = self.registers[b as usize % MAX_VAL];
                        } else {
                            self.registers[a as usize % MAX_VAL] = b % MAX_VAL as u16;
                        }
                        self.pc += 3;
                    },
                    InstructionCode::ADD => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = (
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b} 
                              + if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                            ) % MAX_VAL as u16;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::MULT => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = ((
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL] as u64} else {b as u64} 
                              * if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL] as u64} else {c as u64}
                            ) % MAX_VAL as u64) as u16;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::MOD => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = (
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b} 
                              % if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                            ) % MAX_VAL as u16;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::AND => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = (
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b} 
                              & if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                            ) % MAX_VAL as u16;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::OR => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = (
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b} 
                              | if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                            ) % MAX_VAL as u16;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::EQ => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = {
                                if (
                                    if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b}
                                ) == (
                                    if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                                ) {1} else {0}
                            };
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::GT => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        let c = i.operands.2;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = {
                                if (
                                    if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b}
                                ) > (
                                    if c >= MAX_VAL as u16 {self.registers[c as usize % MAX_VAL]} else {c}
                                ) {1} else {0}
                            };
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 4;
                    },
                    InstructionCode::NOT => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = !(
                                if b >= MAX_VAL as u16 {self.registers[b as usize % MAX_VAL]} else {b}
                            ) & 0x7fff;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 3;
                    },
                    InstructionCode::PUSH => {
                        let a = i.operands.0;
                        if a >= MAX_VAL as u16 {
                            self.stack.push(self.registers[a as usize % MAX_VAL]);
                        } else {
                            self.stack.push(a);
                        }
                        self.pc += 2;
                    },
                    InstructionCode::POP => {
                        let a = i.operands.0;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] = self.stack.pop().unwrap();
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 2;
                    },
                    InstructionCode::RMEM => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if a >= MAX_VAL as u16 {
                            self.registers[a as usize % MAX_VAL] =
                                if b >= MAX_VAL as u16 {
                                    self.memory[self.registers[b as usize % MAX_VAL] as usize]
                                } else {
                                    self.memory[b as usize]
                                }
                            ;
                        } else {
                            error!("Operand [a] doesn't resolve into a register?");
                        }
                        self.pc += 3;
                    },
                    InstructionCode::WMEM => {
                        let a = i.operands.0;
                        let b = i.operands.1;
                        if a >= MAX_VAL as u16 {
                            self.memory[self.registers[a as usize % MAX_VAL] as usize] =
                                if b >= MAX_VAL as u16 {
                                    self.registers[b as usize % MAX_VAL]
                                } else {
                                    b
                                }
                            ;
                        } else {
                            self.memory[a as usize] = 
                                if b >= MAX_VAL as u16 {
                                    self.registers[b as usize % MAX_VAL]
                                } else {
                                    b
                                }                    
                            ;
                        }
                        self.pc += 3;
                    },
                }
            },
            Err(_i) => {
                error!("Unknown instruction code. = {}", self.memory[self.pc]);
                std::process::exit(0);
            }
        }
        debug!("PC: {}", self.pc);
    }

    pub fn disassemble(&mut self, filename: String) {
        info!("disassemble()");
        let f = File::create(filename).unwrap();
        let mut file = LineWriter::new(f);
        let mut pc: usize = 0;
        while pc < self.memory.len() {
            match Instruction::parse(&self.memory, pc) {
                Ok(i) => {
                    match i.operator.clone() {
                        InstructionCode::NOOP | InstructionCode::HALT | InstructionCode::RET => {
                            writeln!(file, "{:<5} {:<5?}", pc, i.operator).unwrap();
                            pc += 1;
                        },
                        InstructionCode::OUT => {
                            writeln!(file, "{:<5} {:<5?}  {:<5}", pc, i.operator, 
                                if i.operands.0 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.0 % MAX_VAL as u16).to_string());
                                    s
                                } else {
                                    if i.operands.0 == 10 {
                                        "\\n".to_string()
                                    } else {
                                        (i.operands.0 as u8 as char).to_string()
                                    }
                                }).unwrap();
                            pc += 2;
                        },
                        InstructionCode::IN | InstructionCode::JMP | InstructionCode::CALL | InstructionCode::PUSH | InstructionCode::POP => {
                            writeln!(file, "{:<5} {:<5?}  {:<5}", pc, i.operator, 
                                if i.operands.0 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.0 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.0.to_string()}
                            ).unwrap();
                            pc += 2;
                        },
                        InstructionCode::JT | InstructionCode::JF | InstructionCode::SET | InstructionCode::NOT | 
                        InstructionCode::RMEM | InstructionCode::WMEM => {
                            writeln!(file, "{:<5} {:<5?}  {:<5} {:<5}", pc, i.operator, 
                                if i.operands.0 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.0 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.0.to_string()},
                                if i.operands.1 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.1 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.1.to_string()}
                            ).unwrap();
                            pc += 3;
                        },
                        InstructionCode::ADD | InstructionCode::MULT | InstructionCode::MOD | 
                        InstructionCode::AND | InstructionCode::OR | InstructionCode::EQ | InstructionCode::GT => {
                            writeln!(file, "{:<5} {:<5?}  {:<5} {:<5} {:<5}", pc, i.operator, 
                                if i.operands.0 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.0 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.0.to_string()},
                                if i.operands.1 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.1 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.1.to_string()},
                                if i.operands.2 >= MAX_VAL as u16 {
                                    let mut s = "R".to_string();
                                    s.push_str(&(i.operands.2 % MAX_VAL as u16).to_string());
                                    s
                                } else {i.operands.2.to_string()}
                            ).unwrap();

                            pc += 4;
                        },
                    }
                },
                Err(_) => {
                    // warn!("Unknown instruction: {:?}", i);
                    writeln!(file, "{:<5} {:<5}", pc, self.memory[pc]).unwrap();
                    pc += 1;
                }
            }
        }
    }

    pub fn execute_until_done(&mut self) {
        info!("execute_until_done()");

        self.pc = 0;
        while !self.stopped.load(Ordering::SeqCst) {
            if !self.paused.load(Ordering::SeqCst) {
                self.execute_once();
            } else {
                thread::sleep(Duration::from_millis(250));
            }
            thread::sleep(Duration::from_millis(self.step_delay));
        }
        debug!("Execution has stopped.");
    }

    #[allow(dead_code)]
    pub fn print_memory(&self) {
        for e in 0..self.memory.len() {
            print!("{:5} ", self.memory[e]);
            if e % 30 == 29 {
                println!();
            }
        }
        println!();
    }

    #[allow(dead_code)]
    pub fn insert_buffer(&mut self, s: String) {
        self.buffer.push_str(&*s);
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn unpause(&mut self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn is_stopped(&mut self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    fn report_error(&mut self, i: Instruction, disassemble: bool) {
        error!("PC        => {:?}", self.pc);
        error!("MEM[PC]   => {:?}", self.memory[self.pc]);
        error!("I         => {:?}", i);
        error!("REGISTERS => {:?}", self.registers);
        error!("STACK     => {:?}", self.stack);
        if disassemble {
            self.disassemble("dump1.asm".to_string());        
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        match env_logger::try_init() {
            Ok(_) => {
                info!("Initializing logging...");
            },
            Err(_) => {

            }
        }
    }

    #[test]
    fn test_instruction_parsing() {
        init();

        let halt = vec![0u16];

        assert_eq!(
            Instruction::parse(&halt, 0), 
            Ok(Instruction {
                operator: InstructionCode::HALT,
                operands: (0u16, 0u16, 0u16)
            })
        );

        let outp = vec![19u16, 42u16];

        assert_eq!(
            Instruction::parse(&outp, 0), 
            Ok(Instruction {
                operator: InstructionCode::OUT,
                operands: (42u16, 0u16, 0u16)
            })
        );

        let inp = vec![20u16, 42u16];

        assert_eq!(
            Instruction::parse(&inp, 0).unwrap(), 
            Instruction {
                operator: InstructionCode::IN,
                operands: (42u16, 0u16, 0u16)
            }
        );

        let noop = vec![21u16];

        assert_eq!(
            Instruction::parse(&noop, 0).unwrap(), 
            Instruction {
                operator: InstructionCode::NOOP,
                operands: (0u16, 0u16, 0u16)
            }
        );
    }

    #[test]
    fn test_vm_creation() {
        init();

        let code = vec![0u8, 0u8];

        let mut vm = Vm::new(code, 4);

        assert_false!(
            vm.is_stopped()
        );

        vm.execute_once();

        assert_true!(
            vm.is_stopped()
        );

    }
}