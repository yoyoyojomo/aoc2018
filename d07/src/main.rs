use std::collections::HashSet;
use std::error::Error;
use std::io::{self, BufRead};
use std::iter::FromIterator;
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

fn skip_str<T: Iterator<Item = char>>(it: &mut T, s: &str) -> Result<()> {
    for c in s.chars() {
        match it.next() {
            Some(x) if x == c => (),
            _ => return Err("failed parse".into()),
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut deps = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let mut line_it = line.chars();
        skip_str(&mut line_it, "Step ")?;
        let src = line_it
            .next()
            .ok_or_else(|| Box::<Error>::from("missing step"))?;
        skip_str(&mut line_it, " must be finished before step ")?;
        let dst = line_it
            .next()
            .ok_or_else(|| Box::<Error>::from("missing step"))?;
        skip_str(&mut line_it, " can begin.")?;
        if let Some(_) = line_it.next() {
            return Err("extra input".into());
        }
        deps.push((src, dst));
    }

    part1(deps.clone())?;
    part2(deps)?;
    Ok(())
}

struct TopologicalScheduler {
    deps: Vec<(char, char)>,
    sinks: HashSet<char>,
}

impl TopologicalScheduler {
    fn new(deps: Vec<(char, char)>) -> Self {
        let sinks = deps.iter().map(|&(_, d)| d).collect();
        Self { deps, sinks }
    }

    fn frontier(&self) -> HashSet<char> {
        if self.deps.is_empty() {
            self.sinks.clone()
        } else {
            let srcs: HashSet<char> = self.deps.iter().map(|&(s, _)| s).collect();
            let dsts: HashSet<char> = self.deps.iter().map(|&(_, d)| d).collect();
            srcs.difference(&dsts).cloned().collect()
        }
    }

    fn peek(&self) -> Option<char> {
        let mut frontier: Vec<char> = Vec::from_iter(self.frontier());
        frontier.sort();
        match frontier.as_slice() {
            [] => None,
            // Rust doesn't seem to yet support destructuring unknown length slices.
            x => Some(x[0]),
        }
    }

    fn pop(&mut self, val: char) {
        self.deps.retain(|&(src, _)| src != val);
        self.sinks.remove(&val);
    }
}

fn part1(deps: Vec<(char, char)>) -> Result<()> {
    let mut topo = TopologicalScheduler::new(deps);
    while let Some(next) = topo.peek() {
        print!("{}", next);
        topo.pop(next);
    }
    println!();
    Ok(())
}

fn work_time(work: char) -> u32 {
    60 + (work as u32 - b'A' as u32 + 1)
}

fn part2(deps: Vec<(char, char)>) -> Result<()> {
    let mut topo = TopologicalScheduler::new(deps);
    let mut workers: Vec<(u32, char)> = Vec::new();
    let mut now = 0;
    loop {
        // Finish work.
        workers = workers.iter().cloned().filter(|&(ready, work)| {
            if ready <= now {
                topo.pop(work);
                false
            } else {
                true
            }
        }).collect();

        // Find next work not already scheduled, if any.
        let working: HashSet<char> = workers.iter().map(|&(_, w)| w).collect();
        let mut frontier = Vec::from_iter(topo.frontier().difference(&working).cloned());
        frontier.sort();
        if !frontier.is_empty() && workers.len() < 5 {
            let work = frontier[0];
            workers.push((now + work_time(work), work));
        } else {
            match workers.iter().map(|&(r, _)| r).min() {
                Some(ready) => now = ready,
                None => break,
            }
        }
    }
    println!("{}", now);
    Ok(())
}
