use std::fmt::{self, Display, Formatter};

/// IMPORTANT: this corresponds to the node identifier in the save file, which skips over broken
/// nodes. The node index in ComputeGrid may be different.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaveFileNodeId(pub u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Port {
    UP, DOWN, LEFT, RIGHT, ANY, LAST,
}

impl Display for Port {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&format!("{:?}", self))
    }
}

impl Port {
    pub fn opposite(self) -> Port {
        match self {
            Port::UP => Port::DOWN,
            Port::DOWN => Port::UP,
            Port::LEFT => Port::RIGHT,
            Port::RIGHT => Port::LEFT,
            _ => panic!("can't get opposite of {:?}", self)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Register {
    ACC, NIL,
    // excludes BAK because it cannot be addressed
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Src {
    Register(Register),
    Port(Port),
    Immediate(i16),
}

impl Display for Src {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Src::Register(r) => write!(f, "{:?}", r),
            Src::Port(p) => write!(f, "{:?}", p),
            Src::Immediate(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dst {
    Register(Register),
    Port(Port),
}

impl Display for Dst {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Dst::Register(r) => write!(f, "{:?}", r),
            Dst::Port(p) => write!(f, "{:?}", p),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    MOV(Src, Dst),
    SWP,
    SAV,
    ADD(Src),
    SUB(Src),
    NEG,
    JMP(String),
    JEZ(String),
    JNZ(String),
    JGZ(String),
    JLZ(String),
    JRO(Src),
    HCF,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&match self {
            Instruction::NOP | Instruction::SWP | Instruction::SAV | Instruction::NEG
                    | Instruction::HCF => {
                format!("{:?}", self)
            }
            Instruction::MOV(src, dst) => format!("MOV {}, {}", src, dst),
            Instruction::ADD(src) => format!("ADD {}", src),
            Instruction::SUB(src) => format!("SUB {}", src),
            Instruction::JMP(l) => format!("JMP {}", l),
            Instruction::JEZ(l) => format!("JEZ {}", l),
            Instruction::JNZ(l) => format!("JNZ {}", l),
            Instruction::JGZ(l) => format!("JGZ {}", l),
            Instruction::JLZ(l) => format!("JLZ {}", l),
            Instruction::JRO(s) => format!("JRO {}", s),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramItem {
    Instruction(Instruction),
    Label(String),
    Breakpoint,
}
