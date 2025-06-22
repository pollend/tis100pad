use std::env;
use std::fs;
use std::process;
use tis100::assembly::Nodes;

fn main() {
    stderrlog::new()
        .verbosity(4)
        .init()
        .unwrap();

    let path = env::args_os().nth(1).unwrap_or_else(|| {
        eprintln!("usage: {} <save file path>", env::args().next().unwrap());
        process::exit(1);
    });

    let input = fs::read(path).expect("failed to read file");

    match tis100::assembly::parse_save_file(&input) {
        Ok(map) => {
            print_program(&map);
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
            eprintln!("parse error at {}:{} (offset {})", line, col, pos);
            eprintln!("parsed input up to that point:");
            print_program(&nodes);
        }
    }
}

fn print_program(nodes: &Nodes) {
    for (node_id, items) in nodes {
        println!("{:?}", node_id);
        for item in items {
            println!("\t{:?}", item);
        }
    }
}
