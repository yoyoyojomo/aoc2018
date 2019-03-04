use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

const PADDING: usize = 100;

fn parse_transition(mut line: Vec<u8>) -> Result<(Vec<u8>, u8)> {
    if line.len() != 10 || &line[5..9] != b" => " {
        return Err("transition does not parse".into());
    }
    let to = line[9];
    line.truncate(5);
    Ok((line, to))
}

fn evolve(state: &[u8], transitions: &BTreeMap<Vec<u8>, u8>) -> Result<Vec<u8>> {
    let mut result = vec![b'.'; state.len()];
    for i in 2..state.len() - 2 {
        result[i] = *transitions
            .get(&state[i - 2..i + 3])
            .ok_or("no transition found")?;
    }
    if result[2] == b'#' || result[result.len() - 3] == b'#' {
        return Err("insufficient padding".into());
    }
    Ok(result)
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

    let mut state = vec![b'.'; 2 * PADDING + initial.len()];
    state[PADDING..PADDING + initial.len()].copy_from_slice(&initial);
    for _ in 0..20 {
        state = evolve(&state, &transitions)?;
    }

    let sum: i32 = state
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c == b'#')
        .map(|(i, _)| (i as i32) - (PADDING as i32))
        .sum();
    println!("{}", sum);

    Ok(())
}
