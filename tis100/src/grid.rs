use crate::assembly::Nodes;
use crate::compute::ComputeNode;
use crate::instr::{Port, ProgramItem};
use crate::io::{InputNode, OutputNode, VerifyState};
use crate::node::{Node, NodeType, NodeOps, BrokenNode, StepResult};
use crate::puzzles::{Puzzle, PUZZLE_WIDTH, PUZZLE_HEIGHT, VIZ_WIDTH, VIZ_HEIGHT};
use crate::stack::StackNode;
use crate::visualization::VisualizationNode;

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ComputeGrid {
    nodes: Vec<Node>, // arranged in row-major order
    external: BTreeMap<(usize, Port), Node>,
    row_width: usize,
}

impl ComputeGrid {
    pub fn from_puzzle(p: Puzzle) -> ComputeGrid {
        let mut nodes = Vec::with_capacity(PUZZLE_WIDTH * PUZZLE_HEIGHT);
        for idx in 0 .. PUZZLE_WIDTH * PUZZLE_HEIGHT {
            let node = if p.bad_nodes.contains(&idx) {
                Node::new(NodeType::Broken(BrokenNode))
            } else if p.stack_nodes.contains(&idx) {
                Node::new(NodeType::Stack(StackNode::default()))
            } else {
                Node::new(NodeType::Compute(ComputeNode::default()))
            };
            nodes.push(node);
        }

        let mut external = BTreeMap::new();
        for ((id, port), data) in p.inputs.into_iter() {
            external.insert((id, port), Node::new(NodeType::Input(InputNode::new(data))));
        }
        for ((id, port), data) in p.outputs.into_iter() {
            external.insert((id, port), Node::new(NodeType::Output(OutputNode::new(data))));
        }
        for ((id, port), data) in p.visual.into_iter() {
            external.insert(
                (id, port),
                Node::new(
                    NodeType::Visualization(
                        VisualizationNode::new(data, VIZ_WIDTH, VIZ_HEIGHT))));
        }

        ComputeGrid {
            nodes,
            external,
            row_width: PUZZLE_WIDTH,
        }
    }

    pub fn program_node(&mut self, idx: usize, program_items: impl IntoIterator<Item=ProgramItem>)
        -> bool
    {
        self.nodes[idx].program_node(program_items.into_iter())
    }

    pub fn program_nodes(&mut self, nodes: Nodes) {
        let mut offset = 0;
        for (id, asm) in nodes {
            if log_enabled!(log::Level::Debug) {
                debug!("Save file node {}:", id.0);
                for inst in &asm {
                    debug!("\t{:?}", inst);
                }
            }

            let mut asm_iter = asm.into_iter();
            loop {
                let idx = id.0 as usize + offset;
                let programmed = self.program_node(idx, &mut asm_iter);
                if programmed {
                    debug!("\tprogrammed node {}", idx);
                    break;
                } else {
                    // encountered a broken node; continue to the next one.
                    offset += 1;
                }
            }
        }
    }

    fn compute_nodes(&self) -> impl Iterator<Item=&ComputeNode> + '_ {
        self.nodes.iter()
            .filter_map(|node| match node.inner {
                NodeType::Compute(ref c) => Some(c),
                _ => None,
            })
    }

    pub fn count_programmed_nodes(&self) -> usize {
        self.compute_nodes()
            .filter(|node| !node.instructions.is_empty())
            .count()
    }

    pub fn count_instructions(&self) -> usize {
        self.compute_nodes()
            .fold(0, |acc, node| acc + node.instructions.len())
    }

    pub fn step(&mut self) -> Option<bool> {
        self.read();
        self.compute();
        self.write();
        self.advance();

        let mut all_verified = true;
        for node in self.external.values() {
            match node.verify_state() {
                Some(VerifyState::Finished) => (),
                Some(VerifyState::Failed) => {
                    return Some(false);
                }
                Some(VerifyState::Blocked) | Some(VerifyState::Okay) => { all_verified = false; }
                None => ()
            }
        }

        if all_verified {
            Some(true)
        } else {
            None
        }
    }

    fn get_neighbor(&mut self, idx: usize, port: Port) -> Option<(&mut Node, Option<usize>)> {
        if let Some(node) = self.external.get_mut(&(idx, port)) {
            Some((node, None))
        } else {
            match port {
                Port::UP => {
                    if idx >= self.row_width {
                        let n = idx - self.row_width;
                        Some((&mut self.nodes[n], Some(n)))
                    } else {
                        None
                    }
                }
                Port::LEFT => {
                    if idx > 0 {
                        let n = idx - 1;
                        Some((&mut self.nodes[n], Some(n)))
                    } else {
                        None
                    }
                }
                Port::RIGHT => {
                    if idx < self.nodes.len() - 1 {
                        let n = idx + 1;
                        Some((&mut self.nodes[n], Some(n)))
                    } else {
                        None
                    }
                }
                Port::DOWN => {
                    let n = idx + self.row_width;
                    if n < self.nodes.len() {
                        Some((&mut self.nodes[n], Some(n)))
                    } else {
                        None
                    }
                }
                _ => panic!("can't get neighbor {:?} ya dingus", port)
            }
        }
    }

    pub fn read(&mut self) {
        debug!("begin READ step");

        for idx in 0 .. self.nodes.len() {
            if let NodeType::Broken(_) = self.nodes[idx].inner {
                continue;
            }

            // get readable values from neighbors
            let mut avail_reads = vec![];

            let mut add_value_from = |attached_port: Port| {
                if let Some((node, _idx)) = self.get_neighbor(idx, attached_port) {
                    if let Some((port, val)) = node.pending_output() {
                        if port == attached_port.opposite() || port == Port::ANY {
                            avail_reads.push((attached_port, Some(val)));
                        }
                    }
                }
            };

            // The order is important because it affects which port completes first for an ANY read.
            add_value_from(Port::LEFT);
            add_value_from(Port::RIGHT);
            add_value_from(Port::UP);
            add_value_from(Port::DOWN);

            // Step the node!

            debug!("node {}", idx);
            let result = self.nodes[idx].read(avail_reads.as_mut_slice());
            if result != StepResult::Nothing {
                debug!("  result: {:?}", result);
            }

            for (port, val) in &avail_reads {
                if val.is_none() {
                    // the value was taken
                    let (node, idx) = self.get_neighbor(idx, *port).unwrap();
                    if let Some(idx) = idx {
                        debug!("completing write for node {}", idx);
                    } else {
                        debug!("completing write for {} node", node.type_name());
                    }
                    node.complete_write(*port);
                }
            }
        }

        // Now step the I/O nodes
        for ((idx, rel_port), ref mut node) in &mut self.external {
            let mut avail_reads = vec![];

            if let Some((dest_port, value)) = self.nodes[*idx].pending_output() {
                if dest_port == Port::ANY || dest_port == *rel_port {
                    avail_reads.push((rel_port.opposite(), Some(value))); // port doesn't matter actually
                }
            }

            debug!("{} port", node.type_name());
            let result = node.read(avail_reads.as_mut_slice());
            if result != StepResult::Nothing {
                debug!("  result: {:?}", result);
            }

            if let Some((_port, val)) = avail_reads.first() {
                if val.is_none() {
                    // the value was taken
                    debug!("completing write for node {}", idx);
                    self.nodes[*idx].complete_write(*rel_port);
                }
            }
        }
    }

    fn compute(&mut self) {
        debug!("begin COMPUTE step");
        for idx in 0 .. self.nodes.len() {
            if let NodeType::Broken(_) = self.nodes[idx].inner {
                continue;
            }

            let result = self.nodes[idx].compute();
            if result != StepResult::Nothing {
                debug!("node {}: {:?}", idx, result);
            }
        }
        for node in self.external.values_mut() {
            node.compute();
        }
    }

    fn write(&mut self) {
        debug!("begin WRITE step");

        for idx in 0 .. self.nodes.len() {
            if let NodeType::Broken(_) = self.nodes[idx].inner {
                continue;
            }

            debug!("node {}", idx);
            let result = self.nodes[idx].write();
            if result != StepResult::Nothing {
                debug!("  result: {:?}", result);
            }
        }

        for node in self.external.values_mut() {
            debug!("{} port", node.type_name());
            let result = node.write();
            if result != StepResult::Nothing {
                debug!("  result: {:?}", result);
            }
        }
    }

    fn advance(&mut self) {
        debug!("begin ADVANCE step");
        for idx in 0 .. self.nodes.len() {
            let result = self.nodes[idx].advance();
            if result != StepResult::Nothing {
                debug!("node {}: {:?}", idx, result);
            }
        }
        for node in self.external.values_mut() {
            node.advance();
        }
    }

    pub fn print(&self) {
        let p_inst = |idx: usize, i: usize| {
            if let NodeType::Compute(c) = &self.nodes[idx].inner {
                if let Some(inst) = c.instructions.get(i) {
                    if c.pc == i {
                        print!(">");
                    } else {
                        print!(" ");
                    }
                    print!("{:16}", inst);
                } else {
                    print!(" {:16}", "");
                }
            } else {
                print!(" {:16}", "");
            }
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        };

        macro_rules! p {
            ($idx:expr, reg $r:ident) => {
                if let NodeType::Compute(c) = &self.nodes[$idx].inner {
                    print!("{:5}", c.$r);
                } else {
                    print!("{:5}", "");
                }
            };
            ($idx:expr, step) => {
                let node = &self.nodes[$idx];
                if let NodeType::Compute(_) = &node.inner {
                    print!("{:5}", node.step);
                } else {
                    print!("{:5}", "");
                }
            };
            ($idx:expr, pending port) => {
                let node = &self.nodes[$idx];
                if let NodeType::Compute(_) = &node.inner {
                    if let Some((port, _val)) = &node.pending_output {
                        print!("{:5}", port);
                    } else {
                        print!("{:5}", "");
                    }
                } else {
                    print!("{:5}", "");
                }
            };
            ($idx:expr, pending value) => {
                let node = &self.nodes[$idx];
                if let NodeType::Compute(_) = &node.inner {
                    if let Some((_port, val)) = &node.pending_output {
                        print!("{:5}", val);
                    } else {
                        print!("{:5}", "");
                    }
                } else {
                    print!("{:5}", "");
                }
            };
        }

        //       "|>MOV RIGHT, RIGHT | RIGHT |  |>MOV RIGHT, RIGHT | RIGHT |  |>MOV RIGHT, RIGHT | RIGHT |  |>MOV RIGHT, RIGHT | RIGHT |");
        println!("+------------------+-------+  +------------------+-------+  +------------------+-------+  +------------------+-------+");

        for (start, end) in [(0,3), (4,7), (8,11)].iter().cloned() {
            let endln = |idx| {
                if idx != end {
                    print!(" |  ");
                } else {
                    println!(" |");
                }
            };
            let block_text = |iidx, text| {
                for idx in start ..= end {
                    print!("|");
                    p_inst(idx, iidx);
                    print!(" | {:5}", text);
                    endln(idx);
                }
            };
            let block_sep = |iidx| {
                for idx in start ..= end {
                    print!("|");
                    p_inst(idx, iidx);
                    if idx != end {
                        print!(" |-------|  ");
                    } else {
                        println!(" |-------|");
                    }
                }
            };

            macro_rules! block_info {
                ($iidx:expr, $($stuff:tt)*) => {
                    for idx in start ..= end {
                        print!("|");
                        p_inst(idx, $iidx);
                        print!(" | ");
                        p!(idx, $($stuff)*);
                        endln(idx);
                    }
                };
            }

            block_text(  0, " ACC ");
            block_info!( 1, reg acc);
            block_sep(   2);
            block_text(  3," BAK ");
            block_info!( 4, reg bak);
            block_sep(   5);
            block_text(  6, "LAST ");
            block_info!( 7, reg last);
            block_sep(   8);
            block_text(  9, "MODE ");
            block_info!(10, step);
            block_sep(  11);
            block_text( 12, "PENDG");
            block_info!(13, pending port);
            block_info!(14, pending value);
            println!("+------------------+-------+  +------------------+-------+  +------------------+-------+  +------------------+-------+");
            println!();
            if end != 11 {
                println!("+------------------+-------+  +------------------+-------+  +------------------+-------+  +------------------+-------+");
            }
        }
    }
}
