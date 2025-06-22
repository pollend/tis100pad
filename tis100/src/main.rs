#![deny(rust_2018_idioms)]

#[macro_use] extern crate log;

use clap::Parser;
use rand::SeedableRng;

use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short='d', long="debug")]
    debug: bool,

    #[arg(short='v', long="verbose", action=clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short='p', long="puzzle")]
    puzzle_num: Option<String>,

    savefile_path: PathBuf,
}

fn main() {
    println!("TESSELLATED INTELLIGENCE SYSTEMS TIS-100 BIOS V2.0-R");
    println!("COPYRIGHT (C) 2018, WILLIAM R. FRASER");

    let mut args = Args::parse();
    if args.debug {
        if args.verbose != 0 {
            eprintln!("warning: debug flag overrides verbose");
        }
        args.verbose = 4;
    }
    let puzzle_num = args.puzzle_num
        .take()
        .unwrap_or_else(||
            args.savefile_path.file_name().unwrap()
                .to_str().unwrap()
                .split('.')
                .next()
                .unwrap()
                .to_owned());

    stderrlog::new()
        .verbosity(usize::from(args.verbose))
        .init()
        .unwrap();

    let input = fs::read(&args.savefile_path)
        .unwrap_or_else(|e| {
            error!("Failed to read {:?}: {}", args.savefile_path, e);
            exit(2);
        });

    let r = rand_chacha::ChaChaRng::from_seed([0;32]);
    let p = tis100::puzzles::get_puzzle(&puzzle_num, r)
        .unwrap_or_else(|| {
            eprintln!("Unknown puzzle number {:?}", puzzle_num);
            exit(1);
        });

    println!(" - SEGMENT {}: \"{}\" -", puzzle_num, p.name);

    let mut grid = tis100::grid::ComputeGrid::from_puzzle(p);

    match tis100::assembly::parse_save_file(&input) {
        Ok(nodes) => {
            grid.program_nodes(nodes);
        }
        Err((remaining, nodes)) => {
            let pos = input.len() - remaining.len();
            let (line, col) = input.iter()
                .take(pos)
                .fold((1, 0), |(mut line, mut col), byte| {
                    if *byte == b'\n' {
                        line += 1;
                        col = 0;
                    } else {
                        col += 1;
                    }
                    (line, col)
                });
            error!("parse error at {}:{} (offset {})", line, col, pos);
            error!("parsed input up to that point: {:?}", nodes);
            exit(1);
        }
    }

    let num_nodes = grid.count_programmed_nodes();
    let num_instructions = grid.count_instructions();

    println!("{} nodes programmed", num_nodes);
    println!("{} total instructions", num_instructions);

    let mut cycle = 1;
    loop {
        match args.verbose {
            0 => (),
            1 => eprint!("\rcycle {}", cycle),
            _ => info!("--- start of cycle {} ---", cycle),
        }
        if let Some(correct) = grid.step() {
            if args.verbose == 1 {
                eprint!("\r");
            }
            if correct {
                print!("correct");
            } else {
                print!("incorrect");
            }
            println!(" solution in {} cycles", cycle);
            break;
        }
        //grid.print();
        cycle += 1;
    }
}
