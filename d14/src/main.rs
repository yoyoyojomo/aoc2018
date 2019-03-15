use std::env;
use std::error::Error;
use std::result;

type Result<T> = result::Result<T, Box<dyn Error>>;

struct Scores {
    scores: Vec<usize>,
    elves: [usize; 2],
    hold: Vec<usize>,
}

impl Iterator for Scores {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if !self.hold.is_empty() {
            return self.hold.pop();
        }

        let sum = self.elves.iter().map(|&e| self.scores[e]).sum::<usize>();
        let result = if sum > 9 {
            assert!(sum < 20);
            self.scores.push(sum / 10);
            self.scores.push(sum % 10);
            self.hold.push(sum % 10);
            sum / 10
        } else {
            self.scores.push(sum);
            sum
        };

        for elf in &mut self.elves {
            *elf = (*elf + self.scores[*elf] + 1) % self.scores.len();
        }
        Some(result)
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let (score0, score1, input) = match &args.as_slice() {
        &[a, b, c] => (a, b, c),
        _ => return Err("expected 3 arguments".into()),
    };

    let score0: usize = score0.parse()?;
    let score1: usize = score1.parse()?;
    let iterations: usize = input.parse()?;

    let scores = Scores {
        scores: vec![score0, score1],
        elves: [0, 1],
        hold: vec![score1, score0],
    };

    for score in scores.skip(iterations).take(10) {
        print!("{}", score);
    }
    println!();

    let scores = Scores {
        scores: vec![score0, score1],
        elves: [0, 1],
        hold: vec![score1, score0],
    };

    let score_pattern: Vec<usize> = input
        .as_bytes()
        .into_iter()
        .map(|c| (c - b'0') as usize)
        .collect();
    let mut matched = 0;

    for (i, score) in scores.enumerate() {
        if score == score_pattern[matched] {
            matched += 1;
            if matched == score_pattern.len() {
                println!("{}", i - matched + 1);
                break;
            }
        } else {
            matched = if score == score_pattern[0] { 1 } else { 0 };
        }
    }

    Ok(())
}
