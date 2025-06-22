use crate::compute::ComputeNode;
use crate::stack::StackNode;
use crate::instr::{Port, ProgramItem};
use crate::io::{InputNode, OutputNode, VerifyState};
use crate::visualization::VisualizationNode;

#[derive(Debug)]
pub struct Node {
    pub inner: NodeType,
    pub step: CycleStep,
    pub pending_output: Option<(Port, i32)>, // port is relative to this node
}

#[derive(Debug)]
pub enum NodeType {
    Broken(BrokenNode),
    Compute(ComputeNode),
    Stack(StackNode),
    Input(InputNode),
    Output(OutputNode),
    Visualization(VisualizationNode),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CycleStep {
    Read, Compute, Write, Advance,
}

impl std::fmt::Display for CycleStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            CycleStep::Read => "READ",
            CycleStep::Compute => "COMP",
            CycleStep::Write => "WRTE",
            CycleStep::Advance => "ADVN",
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StepResult<Output> {
    /// Nothing happened.
    Nothing,

    /// Step successfully completed.
    Okay,

    /// Node has a value to write or read.
    IO(Output),

    /// Node is blocked on some other cycle step.
    Blocked(CycleStep),
}

pub type ReadResult = StepResult<Port>;
pub type ComputeResult = StepResult<()>;
pub type WriteResult = StepResult<(Port, i32)>;
pub type AdvanceResult = StepResult<()>;

pub trait NodeOps {
    fn read(&mut self, _avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        StepResult::Nothing
    }
    fn compute(&mut self) -> ComputeResult {
        StepResult::Nothing
    }
    fn write(&mut self) -> WriteResult {
        StepResult::Nothing
    }
    fn advance(&mut self) -> AdvanceResult {
        StepResult::Nothing
    }
}

#[derive(Debug)]
pub struct BrokenNode;
impl NodeOps for BrokenNode {
    // all default impls
}

impl Node {
    pub fn new(inner: NodeType) -> Node {
        Node {
            inner,
            step: CycleStep::Read,
            pending_output: None,
        }
    }

    pub fn program_node(&mut self, program_items: impl Iterator<Item=ProgramItem>) -> bool {
        match &mut self.inner {
            NodeType::Broken(_) | NodeType::Stack(_) => false,
            NodeType::Compute(comp) => {
                comp.load_assembly(program_items);
                true
            }
            NodeType::Input(_) | NodeType::Output(_) | NodeType::Visualization(_) => {
                panic!("attempted to program an I/O node somehow");
            }
        }
    }

    pub fn verify_state(&self) -> Option<VerifyState> {
        match &self.inner {
            NodeType::Output(out) => Some(out.verified()),
            NodeType::Visualization(v) => Some(v.verified()),
            _ => None,
        }
    }

    pub fn pending_output(&self) -> Option<(Port, i32)> {
        self.pending_output
    }

    pub fn complete_write(&mut self, port: Port) {
        if let NodeType::Stack(_) = self.inner {} else {
            // Stack nodes can have a write completed while they're reading, but other node types
            // should never have this happen.
            assert_eq!(CycleStep::Write, self.step);
        }

        self.step = CycleStep::Advance;
        self.pending_output = None;

        if let NodeType::Compute(node) = &mut self.inner {
            node.complete_write(port);
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.inner {
            NodeType::Broken(_) => "broken",
            NodeType::Compute(_) => "compute",
            NodeType::Stack(_) => "stack",
            NodeType::Input(_) => "input",
            NodeType::Output(_) => "output",
            NodeType::Visualization(_) => "visualization",
        }
    }

    fn inner(&mut self) -> &mut dyn NodeOps {
        match self.inner {
            NodeType::Broken(ref mut n) => n,
            NodeType::Compute(ref mut n) => n,
            NodeType::Stack(ref mut n) => n,
            NodeType::Input(ref mut n) => n,
            NodeType::Output(ref mut n) => n,
            NodeType::Visualization(ref mut n) => n,
        }
    }
}
macro_rules! check_step {
    ($self:expr, $exp:expr) => {
        if $self.step != $exp {
            return StepResult::Blocked($self.step);
        }
    }
}

macro_rules! advance_step {
    ($self:expr, $res:expr, $next:expr) => {
        match $res {
            StepResult::Okay | StepResult::Nothing => {
                $self.step = $next
            }
            _ => ()
        }
    }
}

impl NodeOps for Node {
    fn read(&mut self, avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        check_step!(self, CycleStep::Read);
        let res = self.inner().read(avail_reads);
        advance_step!(self, res, CycleStep::Compute);
        res
    }

    fn compute(&mut self) -> ComputeResult {
        check_step!(self, CycleStep::Compute);
        let res = self.inner().compute();
        advance_step!(self, res, CycleStep::Write);
        res
    }

    fn write(&mut self) -> WriteResult {
        check_step!(self, CycleStep::Write);

        let res = self.inner().write();

        if let StepResult::IO((port, value)) = res {
            self.pending_output = Some((port, value));
        }

        if let NodeType::Stack(_) = self.inner {
            // stack nodes should not get blocked at the Write step
            // advance it to Read (i.e. skip over Advance) instead
            debug!("advancing Stack node to Read");
            self.step = CycleStep::Read;
        }

        advance_step!(self, res, CycleStep::Advance);
        res
    }

    fn advance(&mut self) -> AdvanceResult {
        check_step!(self, CycleStep::Advance);
        let res = self.inner().advance();
        advance_step!(self, res, CycleStep::Read);
        res
    }
}
