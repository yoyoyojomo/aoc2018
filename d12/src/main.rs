use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::iter;
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

struct Pots {
    state: Vec<u8>,
    transitions: BTreeMap<Vec<u8>, u8>,
    offset: usize,
}

impl Pots {
    fn new(mut state: Vec<u8>, transitions: BTreeMap<Vec<u8>, u8>) -> Self {
        state.splice(0..0, iter::repeat(b'.').take(32));
        Self { state, transitions, offset: 32 }
    }

    fn maybe_grow(&mut self) {
        if self.state[2] == b'#' {
            self.state.splice(0..0, iter::repeat(b'.').take(self.offset));
            self.offset *= 2;
        }
        if self.state[self.state.len() - 3] == b'#' {
            self.state.extend(iter::repeat(b'.').take(self.offset));
        }
    }

    fn evolve(&mut self) -> Result<()> {
        self.maybe_grow();
        let mut next = vec![b'.'; self.state.len()];
        for i in 2..self.state.len() - 2 {
            next[i] = *self.transitions
                .get(&self.state[i - 2..i + 3])
                .ok_or("no transition found")?;
        }
        self.state = next;
        Ok(())
    }

    fn trimmed(&self) -> Result<Vec<u8>> {
        Ok(String::from_utf8(self.state.clone())?.trim_matches('.').to_owned().into_bytes())
    }

    fn sum(&self) -> i64 {
        self.state
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c == b'#')
            .map(|(i, _)| (i as i64) - (self.offset as i64))
            .sum()
    }
}

fn parse_transition(mut line: Vec<u8>) -> Result<(Vec<u8>, u8)> {
    if line.len() != 10 || &line[5..9] != b" => " {
        return Err("transition does not parse".into());
    }
    let to = line[9];
    line.truncate(5);
    Ok((line, to))
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().split(b'\n');
    let mut initial = lines.next().ok_or("empty input")??;
    if initial.len() < 15 || initial.drain(..15).collect::<Vec<u8>>() != b"initial state: " {
        return Err("malformed initial state".into());
    }
    if lines.next().ok_or("premature eof")?? != b"" {
        return Err("expected blank line".into());
    }
    let transitions = lines
        .map(|l| parse_transition(l?))
        .collect::<result::Result<BTreeMap<_, _>, _>>()?;

    let mut pots = Pots::new(initial, transitions);
    for _ in 0..20 {
        pots.evolve()?;
    }
    println!("{}", pots.sum());

    let mut last_state = pots.trimmed()?;
    for i in 20..50000000000_u64 {
        pots.evolve()?;
        let state = pots.trimmed()?;
        if state == last_state {
            let sum = pots.sum();
            pots.evolve()?;
            let delta = pots.sum() - sum;
            let projection = sum + delta * (50000000000_u64 - i - 1) as i64;
            println!("{}", projection);
            break;
        }
        last_state = state;
    }

    Ok(())
}
