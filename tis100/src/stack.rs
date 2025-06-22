use crate::node::{StepResult, ReadResult, WriteResult, AdvanceResult, NodeOps};
use crate::instr::Port;

#[derive(Debug, Default)]
pub struct StackNode {
    values: Vec<i32>,
}

impl NodeOps for StackNode {
    fn read(&mut self, avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        if let Some((src_port, value)) = avail_reads.get_mut(0).take() {
            let value = value.take().unwrap();
            debug!("stack node read {} from {}", value, src_port);
            self.values.push(value);
            return StepResult::Okay
        }
        StepResult::Nothing // don't return IO because we don't want to get stuck here
    }

    // default impl for compute

    fn write(&mut self) -> WriteResult {
        if let Some(value) = self.values.last() {
            StepResult::IO((Port::ANY, *value))
        } else {
            StepResult::Nothing
        }
    }

    fn advance(&mut self) -> AdvanceResult {
        self.values.pop();
        StepResult::Okay
    }

}
