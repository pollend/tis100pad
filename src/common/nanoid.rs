//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// https://github.com/ai/nanoid

use std::cmp::min;

use rand::{thread_rng, Rng};

pub const URL_SAFE: [char; 64] = [
    '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub struct NanoID<const N: usize>([u8; N]);
impl<const N: usize> NanoID<N> {
    pub const fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl<'a, const N: usize> Into<&'a str> for &'a NanoID<N> {
    fn into(self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

pub fn generate_nanoid<const N: usize>(alphabet: &[char]) -> NanoID<N> {
    assert!(
        alphabet.len() <= u8::max_value() as usize,
        "The alphabet cannot be longer than a `u8` (to comply with the `random` function)"
    );
    let mask = alphabet.len().next_power_of_two() - 1;
    debug_assert!(alphabet.len() <= mask + 1);
    let step_len = min(32, 8 * N / 5);
    let mut bytes: [u8; 32] = [0; 32];
    let mut res: [u8; N] = [0; N];
    let mut index = 0;
    loop {
        thread_rng().fill(&mut bytes[0..step_len]);
        for &b in &bytes[0..step_len] {
            let byte = b as usize & mask;
            if alphabet.len() > byte {
                res[index] = alphabet[byte] as u8;
                index += 1;
                if index == res.len() {
                    return NanoID(res);
                }
            }
        }
    }
}

