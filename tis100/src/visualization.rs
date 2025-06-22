use crate::io::VerifyState;
use crate::instr::Port;
use crate::node::{StepResult, ReadResult, NodeOps};
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum Color {
    Black = 0,
    DarkGray = 1,
    LightGray = 2,
    White = 3,
    Red = 4,
}

#[derive(Debug)]
pub struct VisualizationNode {
    expected: Vec<Color>,
    values: Vec<Color>,
    cursor: Option<(usize, Option<usize>)>,
    width: usize,
    height: usize,
    verified: VerifyState,
}

fn in_bounds(value: i32, size: usize) -> bool {
    value >= 0 && (value as usize) < size
}

impl VisualizationNode {
    pub fn new(expected: Vec<Color>, width: usize, height: usize) -> Self {
        assert_eq!(expected.len(), width * height);
        VisualizationNode {
            expected,
            values: vec![Color::Black; width * height],
            cursor: None,
            width,
            height,
            verified: VerifyState::Blocked,
        }
    }

    pub fn verified(&self) -> VerifyState {
        self.verified
    }

    fn handle_value(&mut self, value: i32) -> VerifyState {
        if value == -1 {
            info!("resetting cursor");
            self.cursor = None;
            return VerifyState::Okay;
        }

        let new_cursor = match self.cursor {
            None => {
                if !in_bounds(value, self.width) {
                    error!("out-of-bounds X value {}", value);
                    return VerifyState::Failed
                }
                info!("cursor X value set to {}", value);
                Some((value as usize, None))
            },
            Some((x, None)) => {
                if !in_bounds(value, self.height) {
                    error!("out-of-bounds Y value {}", value);
                    return VerifyState::Failed;
                }
                info!("cursor Y value set to {}", value);
                Some((x, Some(value as usize)))
            }
            Some((mut x, Some(y))) => {
                if let Some(color) = Color::from_i32(value) {
                    let idx = y * self.width + x;
                    info!("setting {},{} (offset {}) to {:?}", x, y, idx, color);

                    if self.expected[idx] != color {
                        error!("set to wrong color: expected {:?}", self.expected[idx]);
                        // NOTE: it's possible that the correct color could be set later.
                        // The game itself does not abort when a wrong color is set.
                        // (It actually doesn't seem to abort ever due to a visualization node...)
                        return VerifyState::Failed;
                    }

                    self.values[idx] = color;
                } else {
                    error!("invalid color value {}", value);
                    return VerifyState::Failed;
                }

                if x + 1 < self.width {
                    x += 1;
                }
                Some((x, Some(y)))
            }
        };

        if self.values == self.expected {
            info!("all done!");
            return VerifyState::Finished;
        }

        self.cursor = new_cursor;
        VerifyState::Okay
    }
}

impl NodeOps for VisualizationNode {
    fn read(&mut self, avail_reads: &mut [(Port, Option<i32>)]) -> ReadResult {
        if let Some((port, val)) = avail_reads.get_mut(0) {
            let val = val.take().unwrap();
            info!("handling value {} from {}", val, port);
            let state = self.handle_value(val);
            self.verified = state;
            state.as_read_result()
        } else {
            StepResult::IO(Port::ANY)
        }
    }

    // default impls for compute, write, and advance
}
