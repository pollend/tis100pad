use crate::instr::*;
use crate::node::{StepResult, ReadResult, ComputeResult, WriteResult, AdvanceResult, NodeOps};

use std::collections::HashMap;

#[derive(Debug)]
pub struct ComputeNode {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
    pub acc: i32,
    pub bak: i32,
    pub pc: usize,
    pub last: Port,
    pub read_result: Option<i32>,
}

#[derive(Debug)]
pub enum CycleStep {
    Read, Compute, Write, Advance,
}

macro_rules! get_instr {
    ($self:expr) => {
        match $self.instructions.get($self.pc) {
            Some(i) => i,
            None => {
                return StepResult::Nothing;
            }
        }
    };
}

impl Default for ComputeNode {
    fn default() -> Self {
        ComputeNode {
            instructions: vec![],
            labels: HashMap::new(),
            acc: 0,
            bak: 0,
            pc: 0,
            last: Port::LAST, // this value invokes nasal demons
            read_result: None,
        }
    }
}

impl ComputeNode {
    pub fn load_assembly(&mut self, items: impl Iterator<Item=ProgramItem>) {
        for item in items {
            match item {
                ProgramItem::Instruction(i) => {
                    self.instructions.push(i);
                }
                ProgramItem::Label(s) => {
                    self.labels.insert(s, self.instructions.len());
                }
                ProgramItem::Breakpoint => (), // TODO
            }
        }

        for instr in &self.instructions {
            match instr {
                Instruction::JMP(l) | Instruction::JEZ(l) | Instruction::JNZ(l)
                    | Instruction::JGZ(l) | Instruction::JLZ(l) =>
                {
                    if !self.labels.contains_key(l) {
                        panic!("instruction references undefined label {:?}", l);
                    }
                },
                _ => ()
            }
        }
    }

    pub fn complete_write(&mut self, port: Port) {
        if matches!(self.instructions.get(self.pc),
            Some(Instruction::MOV(_src, Dst::Port(Port::ANY))))
        {
            self.last = port;
        }
    }
}

impl NodeOps for ComputeNode {
    fn read(&mut self, avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        let instr = get_instr!(self);
        trace!("{}", instr);

        let src = match instr {
            Instruction::MOV(src, _dst) => src,
            Instruction::ADD(src) => src,
            Instruction::SUB(src) => src,
            Instruction::JRO(src) => src,
            _ => {
                trace!("no read needed");
                return StepResult::Nothing;
            }
        };

        self.read_result = Some(match src {
            Src::Register(Register::ACC) => self.acc,
            Src::Register(Register::NIL) => 0,
            Src::Immediate(value) => i32::from(*value),
            Src::Port(port) => {
                fn read(
                    port: Port,
                    avail_reads: &mut [(Port, Option<i32>)],
                    last: &mut Port,
                ) -> Option<i32> {
                    for (src_port, value) in avail_reads {
                        if port == Port::ANY || port == *src_port {
                            if port == Port::ANY {
                                *last = *src_port;
                            }
                            return value.take();
                        }
                    }
                    None
                }

                let actual_port = if *port == Port::LAST {
                    if self.last == Port::LAST {
                        panic!("attempted to read from unset LAST port!");
                    }
                    trace!("LAST -> {}", self.last);
                    self.last
                } else {
                    *port
                };

                match read(actual_port, avail_reads, &mut self.last) {
                    Some(value) => {
                        trace!("ready value from {}: {}", actual_port, value);
                        value
                    }
                    None => {
                        trace!("waiting for {}", *port);
                        return StepResult::IO(*port);
                    }
                }
            }
        });

        StepResult::Okay
    }

    fn compute(&mut self) -> ComputeResult {
        match get_instr!(self) {
            Instruction::NOP => (),
            Instruction::MOV(_src, _dst) => (),
            Instruction::SWP => {
                std::mem::swap(&mut self.acc, &mut self.bak);
            }
            Instruction::SAV => {
                self.bak = self.acc;
            }
            Instruction::ADD(_src) => {
                self.acc += self.read_result.unwrap();
            }
            Instruction::SUB(_src) => {
                self.acc -= self.read_result.unwrap();
            }
            Instruction::NEG => {
                self.acc = -self.acc;
            }
            Instruction::JMP(_) | Instruction::JEZ(_) | Instruction::JNZ(_) | Instruction::JGZ(_)
                | Instruction::JLZ(_) | Instruction::JRO(_) => (),
            Instruction::HCF => {
                error!("tried to execute a Halt and Catch Fire instruction");
                error!("current node state is {:#?}", self);
                error!("we are panicking now");
                panic!("HCF");
            }
        }

        StepResult::Okay
    }

    fn write(&mut self) -> WriteResult {
        let instr = get_instr!(self);
        trace!("{}", instr);

        if let Instruction::MOV(_src, dst) = instr {
            let val = self.read_result.unwrap();
            match dst {
                Dst::Register(Register::ACC) => { self.acc = val; }
                Dst::Register(Register::NIL) => (),
                Dst::Port(port) => {
                    let actual_port = if *port == Port::LAST {
                        if self.last == Port::LAST {
                            panic!("attempted to write to unset LAST port");
                        }
                        trace!("LAST -> {}", self.last);
                        self.last
                    } else {
                        *port
                    };

                    trace!("writing {} to {}", val, actual_port);
                    return StepResult::IO((actual_port, val));
                }
            }
            StepResult::Okay
        } else {
            trace!("no write needed");
            StepResult::Nothing
        }
    }

    fn advance(&mut self) -> AdvanceResult {
        let instr = get_instr!(self);

        match instr {
            Instruction::JRO(_) => (),
            _ => {
                self.pc += 1;
            }
        }

        match instr {
            Instruction::JMP(label) => { self.pc = self.labels[label]; }
            Instruction::JEZ(label) => if self.acc == 0 { self.pc = self.labels[label]; }
            Instruction::JNZ(label) => if self.acc != 0 { self.pc = self.labels[label]; }
            Instruction::JGZ(label) => if self.acc > 0 { self.pc = self.labels[label]; }
            Instruction::JLZ(label) => if self.acc < 0 { self.pc = self.labels[label]; }
            Instruction::JRO(_src) => {
                let off = self.read_result.unwrap();
                if off < 0 {
                    if (-off) as usize > self.pc {
                        self.pc = 0;
                    } else {
                        self.pc -= (-off) as usize;
                    }
                } else {
                    self.pc += off as usize;
                }

                // As an exception to the normal wrap-around, a JRO out of bounds goes to the
                // last instruction.
                if self.pc >= self.instructions.len() {
                    self.pc = self.instructions.len() - 1;
                }
            }
            _ => ()
        }

        if self.pc >= self.instructions.len() {
            self.pc = 0;
        }

        self.read_result = None;

        StepResult::Okay
    }
}
