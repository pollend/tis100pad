#![deny(rust_2018_idioms)]
#[macro_use] extern crate maplit;

use rand::SeedableRng;

use tis100::grid::ComputeGrid;
use tis100::instr::*;
use tis100::puzzles::Puzzle;

fn rng() -> impl rand::Rng + Clone {
    rand_chacha::ChaChaRng::from_seed([0;32])
}

fn asm(input: &str) -> Vec<ProgramItem> {
    tis100::assembly::program_items(input.as_bytes()).expect("asm parse error")
}

trait VecExt<T> {
    fn single(self) -> T;
}

impl<T> VecExt<T> for Vec<T> {
    fn single(mut self) -> T {
        assert_eq!(1, self.len());
        self.drain(..).next().unwrap()
    }
}

#[test]
fn parse_test() {
    assert_eq!(
        ProgramItem::Instruction(
            Instruction::MOV(
                Src::Port(
                    Port::ANY),
                Dst::Port(
                    Port::LAST))),
        asm("MOV ANY, LAST").single());

    assert_eq!(
        ProgramItem::Instruction(
            Instruction::MOV(
                Src::Immediate(999),
                Dst::Register(
                    Register::NIL))),
        asm("MOV 999, NIL").single());

    assert_eq!(
        ProgramItem::Instruction(
            Instruction::JMP("JEZ".to_owned())),
        asm("JMP JEZ").single());
}

#[test]
fn parse_whitespace_test() {
    assert_eq!(
        ProgramItem::Instruction(
            Instruction::JEZ("0".to_owned())),
        asm("\n\t  # foo\n# bar\nJEZ\t0\n# whatever\n\n\n").single());
}

#[test]
#[should_panic(expected = "asm parse error")]
fn out_of_range_immediate() {
    asm("MOV 1000, ANY");
}

fn run(grid: &mut ComputeGrid, expected_cycles: usize) {
    let mut cycle = 1;
    loop {
        match grid.step() {
            None => { cycle += 1; }
            Some(true) => { break; }
            Some(false) => { panic!("incorrect result on cycle {}", cycle); }
        }
        if cycle > expected_cycles {
            panic!("too many (>{}) cycles", expected_cycles);
        }
    }
    assert_eq!(expected_cycles, cycle);
}

#[test]
fn connectivity_test() {
    let puz = Puzzle {
        name: "test",
        bad_nodes: &[],
        stack_nodes: &[],
        inputs: btreemap! { (0, Port::UP) => vec![1,2,3,4] },
        outputs: btreemap! { (11, Port::DOWN) => vec![1,2,3,4] },
        visual: btreemap! {},
    };
    let mut grid = tis100::grid::ComputeGrid::from_puzzle(puz);

    // Move the value around in a serpentine motion.
    // In
    // → → → ↓
    // ↓ ← ← ←
    // → → → ↓
    //       Out
    grid.program_node(0, asm("MOV ANY, RIGHT"));
    grid.program_node(1, asm("MOV ANY, RIGHT"));
    grid.program_node(2, asm("MOV ANY, RIGHT"));
    grid.program_node(3, asm("MOV ANY, DOWN"));
    grid.program_node(7, asm("MOV ANY, LEFT"));
    grid.program_node(6, asm("MOV ANY, LEFT"));
    grid.program_node(5, asm("MOV ANY, LEFT"));
    grid.program_node(4, asm("MOV ANY, DOWN"));
    grid.program_node(8, asm("MOV ANY, RIGHT"));
    grid.program_node(9, asm("MOV ANY, RIGHT"));
    grid.program_node(10, asm("MOV ANY, RIGHT"));
    grid.program_node(11, asm("MOV ANY, DOWN"));

    run(&mut grid, 20);
}

#[test]
fn connectivity_test_asm() {
    // same as connectivity_test() but with save file text parsing
    let asm = b"\
@0
MOV ANY,RIGHT
@1
MOV ANY,RIGHT
@2
MOV ANY,RIGHT
@3
MOV ANY,DOWN
@4
MOV ANY,DOWN
@5
MOV ANY,LEFT
@6
MOV ANY,LEFT
@7
MOV ANY,LEFT
@8
MOV ANY,RIGHT
@9
MOV ANY,RIGHT
@10
MOV ANY,RIGHT
@11
MOV ANY,DOWN";

    let puzzle = tis100::puzzles::get_puzzle("DBG01", rng()).unwrap();
    let mut grid = tis100::grid::ComputeGrid::from_puzzle(puzzle);
    let nodes = tis100::assembly::parse_save_file(asm).unwrap();
    grid.program_nodes(nodes);

    run(&mut grid, 90);
}

#[test]
fn stack_test() {
    let puz = tis100::puzzles::get_puzzle("DBG02", rng()).unwrap();
    let mut grid = tis100::grid::ComputeGrid::from_puzzle(puz);

    // Move 4 times to the stack node, then 4 times out.
    // In
    // ↔ S - -
    // ↓ - - -
    // ↓ - - -
    // Out
    grid.program_node(0, asm("
        MOV UP,RIGHT\nMOV UP,RIGHT\nMOV UP,RIGHT\nMOV UP,RIGHT
        MOV RIGHT,DOWN\nMOV RIGHT,DOWN\nMOV RIGHT,DOWN\nMOV RIGHT,DOWN"));
    grid.program_node(4, asm("MOV UP, DOWN"));
    grid.program_node(8, asm("MOV UP, DOWN"));

    run(&mut grid, 19);
}
