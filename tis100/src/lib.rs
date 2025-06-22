#![deny(rust_2018_idioms)]

#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate num_derive;

pub mod assembly;
pub mod compute;
pub mod grid;
pub mod instr;
pub mod io;
pub mod node;
pub mod puzzles;
pub mod stack;
pub mod visualization;
