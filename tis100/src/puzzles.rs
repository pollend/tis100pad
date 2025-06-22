use crate::instr::Port;
use crate::visualization::Color;
use rand::Rng;
use std::collections::BTreeMap;

pub const PUZZLE_WIDTH: usize = 4;
pub const PUZZLE_HEIGHT: usize = 3;
pub const INPUT_SIZE: usize = 39;
pub const VIZ_WIDTH: usize = 30;
pub const VIZ_HEIGHT: usize = 18;

#[derive(Debug, Default)]
pub struct Puzzle {
    pub name: &'static str,
    pub bad_nodes: &'static [usize],
    pub stack_nodes: &'static [usize],
    pub inputs: BTreeMap<(usize, Port), Vec<i32>>,
    pub outputs: BTreeMap<(usize, Port), Vec<i32>>,
    pub visual: BTreeMap<(usize, Port), Vec<Color>>,
}

fn random_vec(rng: &mut impl Rng, num: usize, min: i32, max: i32) -> Vec<i32> {
    let range = rand::distributions::Uniform::new_inclusive(min, max);
    rng.sample_iter(&range).take(num).collect()
}

pub fn get_puzzle<R: Rng + Clone + 'static>(number: &str, mut rng: R)
    -> Option<Puzzle>
{
    Some(match number {
        "DBG01" => {
            let values = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            Puzzle {
                name: "[simulator debug] Connectivity Check",
                inputs: btreemap! {
                    (0, Port::UP) => values.clone(),
                },
                outputs: btreemap! {
                    (11, Port::DOWN) => values,
                },
                ..Puzzle::default()
            }
        }
        "DBG02" => {
            Puzzle {
                name: "[simulator debug] Stack Node Check",
                stack_nodes: &[1],
                inputs: btreemap! {
                    (0, Port::UP) => vec![1,2,3,4],
                },
                outputs: btreemap! {
                    (8, Port::DOWN) => vec![4,3,2,1],
                },
                ..Puzzle::default()
            }
        }
        "00150" => {
            let r1 = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            //let r1 = vec![51,62,16,83,61,14,35];
            let r2 = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            //let r2 = vec![68,59,59,49,82,16,45];
            Puzzle {
                name: "Self-Test Diagnostic",
                bad_nodes: &[1, 5, 7, 9],
                inputs: btreemap! {
                    (0, Port::UP) => r1.clone(),
                    (3, Port::UP) => r2.clone(),
                },
                outputs: btreemap! {
                    (8, Port::DOWN) => r1,
                    (11, Port::DOWN) => r2,
                },
                ..Puzzle::default()
            }
        }
        "10981" => {
            let input = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            let output = input.iter().map(|n| n*2).collect();
            Puzzle {
                name: "Signal Amplifier",
                bad_nodes: &[3, 8],
                inputs:  btreemap! { ( 1, Port::UP) => input },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "20176" => {
            let input1 = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            let input2 = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            let (output1, output2) = input1.iter()
                .zip(&input2)
                .map(|(a, b)| (a - b, b - a))
                .unzip();
            Puzzle {
                name: "Differential Converter",
                bad_nodes: &[7],
                inputs: btreemap! {
                    (1, Port::UP) => input1,
                    (2, Port::UP) => input2,
                },
                outputs: btreemap! {
                    ( 9, Port::DOWN) => output1,
                    (10, Port::DOWN) => output2,
                },
                ..Puzzle::default()
            }
        }
        "21340" => {
            let b = |x| if x { 1 } else { 0 };
            let input = random_vec(&mut rng, INPUT_SIZE, -2, 2);
            let (mut output1, mut output2, mut output3) = (vec![], vec![], vec![]);
            for n in &input {
                output1.push(b(*n > 0));
                output2.push(b(*n == 0));
                output3.push(b(*n < 0));
            }
            Puzzle {
                name: "Signal Comparator",
                bad_nodes: &[5, 6, 7],
                inputs: btreemap! { (0, Port::UP) => input },
                outputs: btreemap! {
                    ( 9, Port::DOWN) => output1,
                    (10, Port::DOWN) => output2,
                    (11, Port::DOWN) => output3,
                },
                ..Puzzle::default()
            }
        }
        "22280" => {
            let input1 = random_vec(&mut rng, INPUT_SIZE, -30, 0);
            let input2 = random_vec(&mut rng, INPUT_SIZE, -1, 1);
            let input3 = random_vec(&mut rng, INPUT_SIZE, 0, 30);
            let output = input1.iter()
                .zip(&input3)
                .zip(&input2)
                .map(|((a, b), which)|
                     match *which {
                         -1 => *a,
                          0 => a + b,
                          1 => *b,
                          _ => unreachable!()
                    })
                .collect();
            Puzzle {
                name: "Signal Multiplexer",
                bad_nodes: &[8],
                inputs: btreemap! {
                    (1, Port::UP) => input1,
                    (2, Port::UP) => input2,
                    (3, Port::UP) => input3,
                },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "30647" => {
            let input1 = random_vec(&mut rng, INPUT_SIZE / 3, 10, 100);
            let input2 = random_vec(&mut rng, INPUT_SIZE / 3, 10, 100);
            let mut output = vec![];
            for (a, b) in input1.iter().zip(&input2) {
                output.extend_from_slice(&[*a.min(b), *a.max(b), 0]);
            }
            Puzzle {
                name: "Sequence Generator",
                bad_nodes: &[9],
                inputs: btreemap! {
                    (1, Port::UP) => input1,
                    (2, Port::UP) => input2,
                },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "31904" => {
            let mut input = vec![];
            let mut output1 = vec![];
            let mut output2 = vec![];
            let mut acc = 0;
            let mut len = 0;
            let mut zrand = random_vec(&mut rng, INPUT_SIZE, 0, 3);
            *zrand.last_mut().unwrap() = 0; // make sure it ends with 0.
            for (r, n) in zrand.into_iter()
                    .zip(random_vec(&mut rng, INPUT_SIZE, 10, 100).into_iter()) {
                if r == 0 {
                    input.push(0);
                    output1.push(acc);
                    output2.push(len);
                    acc = 0;
                    len = 0;
                } else {
                    input.push(n);
                    acc += n;
                    len += 1;
                }
            }
            Puzzle {
                name: "Squence Counter",
                bad_nodes: &[3],
                inputs: btreemap! { (1, Port::UP) => input },
                outputs: btreemap! {
                    ( 9, Port::DOWN) => output1,
                    (10, Port::DOWN) => output2,
                },
                ..Puzzle::default()
            }
        }
        "32050" => {
            let mut input = random_vec(&mut rng, INPUT_SIZE, -20, 40);
            input[0] = 0; // alter the first to be zero
            let mut output = vec![0];
            output.extend(input.iter()
                .zip(input.iter().skip(1))
                .map(|(a, b)| if (a - b).abs() >= 10 { 1 } else { 0 }));
            Puzzle {
                name: "Signal Edge Detector",
                bad_nodes: &[8],
                inputs:  btreemap! { ( 1, Port::UP) => input },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "33762" => {
            let (inputs, output): (Vec<_>, Vec<_>) = random_vec(&mut rng, INPUT_SIZE, 0, 3)
                .into_iter()
                .scan([0i32, 0i32, 0i32, 0i32], |last, which| {
                    let out = if last[which as usize] == 1 {
                        last[which as usize] = 0;
                        // output is 0 for a high->low transition
                        0
                    } else {
                        last[which as usize] = 1;
                        // output is the number of the input which went low->high
                        which + 1
                    };
                    Some((*last, out))
                })
                .unzip();
            Puzzle {
                name: "Interrupt Handler",
                bad_nodes: &[8],
                inputs: btreemap! {
                    (0, Port::UP) => inputs.iter().map(|v| v[0]).collect(),
                    (1, Port::UP) => inputs.iter().map(|v| v[1]).collect(),
                    (2, Port::UP) => inputs.iter().map(|v| v[2]).collect(),
                    (3, Port::UP) => inputs.iter().map(|v| v[3]).collect(),
                },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "40196" => {
            let (input, output) = random_vec(&mut rng, INPUT_SIZE, 0, 3)
                .into_iter()
                .zip(random_vec(&mut rng, INPUT_SIZE, 1, 30))
                .scan(0, |zeroes, (zrand, nrand)|
                    if zrand == 0 {
                        *zeroes = 0;
                        Some((nrand, 0))
                    } else {
                        *zeroes += 1;
                        if *zeroes == 3 {
                            *zeroes -= 1;
                            Some((0, 1))
                        } else {
                            Some((0, 0))
                        }
                    })
                .unzip();
            Puzzle {
                name: "Signal Pattern Detector",
                bad_nodes: &[3],
                inputs: btreemap! { (1, Port::UP) => input },
                outputs: btreemap! { (10, Port::DOWN) => output },
                ..Puzzle::default()
            }
        }
        "41427" => {
            let mut input = vec![];
            let mut output1 = vec![999];
            let mut output2 = vec![0];
            for i in 0 .. INPUT_SIZE {
                if i > 0
                    && input.last() != Some(&0)
                    && (i == INPUT_SIZE - 1
                        || 0 == rng.gen_range(0 .. 5))
                {
                    input.push(0);
                    if i != INPUT_SIZE - 1 {
                        output1.push(999);
                        output2.push(0);
                    }
                } else {
                    let value = rng.gen_range(10 .. 100);
                    input.push(value);
                    if Some(&value) < output1.last() {
                        *output1.last_mut().unwrap() = value;
                    }
                    if Some(&value) > output2.last() {
                        *output2.last_mut().unwrap() = value;
                    }
                }
            }

            Puzzle {
                name: "Sequence Peak Detector",
                bad_nodes: &[7],
                inputs: btreemap! {
                    (1, Port::UP) => input,
                },
                outputs: btreemap! {
                    ( 9, Port::DOWN) => output1,
                    (10, Port::DOWN) => output2,
                },
                ..Puzzle::default()
            }
        },
        "42656" => {
            let mut input = random_vec(&mut rng, INPUT_SIZE, 10, 100);
            let mut output = vec![];
            let mut buf: Vec<i32> = vec![];
            for (n, random) in input.iter_mut().zip(random_vec(&mut rng, INPUT_SIZE, 0, 5).into_iter()) {
                if random == 0 {
                    *n = 0;
                    output.extend(buf.drain(..).rev());
                    output.push(0);
                } else {
                    buf.push(*n);
                }
            }
            buf.pop();
            output.extend(buf.drain(..).rev());
            output.push(0);
            *input.last_mut().unwrap() = 0;
            Puzzle {
                name: "Sequence Reverser",
                bad_nodes: &[8],
                stack_nodes: &[2, 9],
                inputs: btreemap! {
                    (1, Port::UP) => input,
                },
                outputs: btreemap! {
                    (10, Port::DOWN) => output,
                },
                ..Puzzle::default()
            }
        }
        "43786" => {
            let input1 = random_vec(&mut rng, INPUT_SIZE, 0, 9);
            let input2 = random_vec(&mut rng, INPUT_SIZE, 0, 9);
            let output = input1.iter().zip(input2.iter())
                .map(|(a, b)| a * b)
                .collect();
            Puzzle {
                name: "Signal Multiplier",
                bad_nodes: &[8],
                stack_nodes: &[4, 7],
                inputs: btreemap! {
                    (1, Port::UP) => input1,
                    (2, Port::UP) => input2,
                },
                outputs: btreemap! {
                    (10, Port::DOWN) => output,
                },
                ..Puzzle::default()
            }
        }
        "50370" => {
            Puzzle {
                name: "Image Test Pattern 1",
                bad_nodes: &[4],
                visual: btreemap! {
                    (10, Port::DOWN) => vec![Color::White; VIZ_WIDTH * VIZ_HEIGHT],
                },
                ..Puzzle::default()
            }
        }
        "51781" => {
            Puzzle {
                name: "Image Test Pattern 2",
                bad_nodes: &[0],
                visual: btreemap! {
                    (10, Port::DOWN) => (0 .. VIZ_HEIGHT)
                        .flat_map(|y| {
                            (0 .. VIZ_WIDTH)
                                .map(move |x| (x, y))
                        })
                        .map(|(x, y)| {
                            if x % 2 == y % 2 {
                                Color::White
                            } else {
                                Color::Black
                            }
                        })
                        .collect(),
                },
                ..Puzzle::default()
            }
        }
        "52544" => {
            let mut input = vec![];
            let mut viz = vec![Color::Black; VIZ_WIDTH * VIZ_HEIGHT];
            'rand: while input.len() < 9*4 {
                let x: i32 = rng.gen_range(0 .. VIZ_WIDTH as i32 - 5);
                let y: i32 = rng.gen_range(0 .. VIZ_HEIGHT as i32 - 5);
                let w: i32 = rng.gen_range(3 .. 6);
                let h: i32 = rng.gen_range(3 .. 6);
                // check for overlap or adjacent filled pixels
                for x in x-1 ..= x+w {
                    for y in y-1 ..= y+h {
                        if y > 0 && x > 0
                            && viz[(y as usize) * VIZ_WIDTH + (x as usize)] != Color::Black
                        {
                            continue 'rand;
                        }
                    }
                }
                // add the rectangle and set the pixels
                input.extend_from_slice(&[x, y, w, h]);
                println!("adding {},{} {}x{}", x, y, w, h);
                for x in x .. x+w {
                    for y in y .. y+h {
                        viz[(y as usize) * VIZ_WIDTH + (x as usize)] = Color::White;
                    }
                }
            }
            Puzzle {
                name: "Exposure Mask Viewer",
                bad_nodes: &[3],
                inputs: btreemap! {
                    (1, Port::UP) => input,
                },
                visual: btreemap! {
                    (10, Port::DOWN) => viz,
                },
                ..Puzzle::default()
            }
        }
        "53897" => {
            let input = random_vec(&mut rng, VIZ_WIDTH, 5, VIZ_HEIGHT as i32);
            let mut viz = vec![Color::Black; VIZ_WIDTH * VIZ_HEIGHT];
            for (x, n) in input.iter().cloned().enumerate() {
                for y in VIZ_HEIGHT - (n as usize) .. VIZ_HEIGHT {
                    viz[y * VIZ_WIDTH + x] = Color::White;
                }
            }
            Puzzle {
                name: "Histogram Viewer",
                bad_nodes: &[8],
                inputs: btreemap! {
                    (1, Port::UP) => input,
                },
                visual: btreemap! {
                    (10, Port::DOWN) => viz,
                },
                ..Puzzle::default()
            }
        }
        _ => return None
    })
}
