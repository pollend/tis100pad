use crate::instr::Port;
use crate::node::{StepResult, ReadResult, WriteResult, AdvanceResult, NodeOps};
use std::fmt::Write;

#[derive(Debug)]
pub struct InputNode {
    values: Vec<i32>,
    pos: usize,
}

impl InputNode {
    pub fn new(values: Vec<i32>) -> Self {
        InputNode {
            values,
            pos: 0,
        }
    }
}

impl NodeOps for InputNode {
    // default impls for read and compute

    fn write(&mut self) -> WriteResult {
        if let Some(value) = self.values.get(self.pos) {
            trace!("writing {}", value);
            StepResult::IO((Port::ANY, *value))
        } else {
            StepResult::Nothing
        }
    }

    fn advance(&mut self) -> AdvanceResult {
        if self.pos < self.values.len() {
            info!("{}", self);
            self.pos += 1;
            StepResult::Okay
        } else {
            StepResult::Nothing
        }
    }
}

impl std::fmt::Display for InputNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for (idx, n) in self.values.iter().enumerate() {
            if idx == self.pos {
                write!(out, "<({})> ", n)?;
            } else {
                write!(out, "{} ", n)?;
            }
        }
        out.pop();
        f.pad(&out)
    }
}

#[derive(Debug)]
pub struct OutputNode {
    values: Vec<i32>,
    pos: usize,
    verified: VerifyState,
}

impl OutputNode {
    pub fn new(values: Vec<i32>) -> Self {
        OutputNode {
            values,
            pos: 0,
            verified: VerifyState::Blocked,
        }
    }

    pub fn verified(&self) -> VerifyState {
        self.verified
    }

    fn do_verify(&mut self, avail_read: &mut Option<&mut (Port, Option<i32>)>) -> VerifyState {
        if self.pos < self.values.len() {
            if let Some((port, val)) = avail_read {
                let received = val.take().unwrap();
                info!("checking value {} from {}", received, port);
                info!("{}", self);
                if received == self.values[self.pos] {
                    info!("value is correct");
                    self.pos += 1;
                    if self.pos == self.values.len() {
                        info!("finished now!");
                        VerifyState::Finished
                    } else {
                        VerifyState::Okay
                    }
                } else {
                    error!("wrong input");
                    VerifyState::Failed
                }
            } else {
                trace!("waiting for input");
                VerifyState::Blocked
            }
        } else {
            trace!("finished");
            VerifyState::Finished
        }
    }
}
impl NodeOps for OutputNode {
    fn read(&mut self, avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        let state = self.do_verify(&mut avail_reads.get_mut(0));
        self.verified = state;
        state.as_read_result()
    }

    // default impls for compute, write, and advance.
}

impl std::fmt::Display for OutputNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for (idx, n) in self.values.iter().enumerate() {
            if idx == self.pos {
                write!(out, "<({})> ", n)?;
            } else {
                write!(out, "{} ", n)?;
            }
        }
        out.pop();
        f.pad(&out)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VerifyState {
    Failed,
    Okay,
    Blocked,
    Finished,
}

impl VerifyState {
    pub fn as_read_result(self) -> ReadResult {
        match self {
            VerifyState::Okay => StepResult::Okay,
            VerifyState::Blocked => StepResult::IO(Port::ANY),
            VerifyState::Finished | VerifyState::Failed => StepResult::Nothing,
        }
    }
}
