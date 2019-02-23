use std::error::Error;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::str::{Chars, FromStr};

type Result<T> = ::std::result::Result<T, Box<Error>>;

// TODO try as a newtype?
struct StrParser<'a> {
    it: Peekable<Chars<'a>>,
}

impl<'a> StrParser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            it: s.chars().peekable(),
        }
    }

    fn done(&mut self) -> bool {
        self.it.peek().is_none()
    }

    fn consume_str(&mut self, s: &str) -> Result<()> {
        let orig_iter = self.it.clone();
        for c in s.chars() {
            if self.it.next() != Some(c) {
                self.it = orig_iter;
                return Err("unexpected parse".into());
            }
        }
        Ok(())
    }

    fn parse_usize(&mut self) -> Result<usize> {
        let mut digits = String::new();
        while let Some(&c) = self.it.peek() {
            if !c.is_numeric() {
                break;
            }
            digits.push(c);
            self.it.next();
        }
        Ok(digits.parse()?)
    }
}

#[derive(Debug)]
enum Action {
    BeginShift { guard: usize },
    Sleep,
    Wake,
}

#[derive(Debug)]
struct Event {
    year: usize,
    month: usize,
    day: usize,
    hour: usize,
    min: usize,
    action: Action,
}

impl FromStr for Event {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Event> {
        let mut parser = StrParser::new(s);
        parser.consume_str("[")?;
        let year = parser.parse_usize()?;
        parser.consume_str("-")?;
        let month = parser.parse_usize()?;
        parser.consume_str("-")?;
        let day = parser.parse_usize()?;
        parser.consume_str(" ")?;
        let hour = parser.parse_usize()?;
        parser.consume_str(":")?;
        let min = parser.parse_usize()?;
        parser.consume_str("] ")?;

        let action = if let Ok(()) = parser.consume_str("Guard #") {
            let guard = parser.parse_usize()?;
            parser.consume_str(" begins shift")?;
            Action::BeginShift { guard }
        } else if let Ok(()) = parser.consume_str("falls asleep") {
            Action::Sleep
        } else if let Ok(()) = parser.consume_str("wakes up") {
            Action::Wake
        } else {
            return Err("unexpected action".into());
        };
        if !parser.done() {
            return Err("trailing input".into());
        }
        Ok(Event {
            year,
            month,
            day,
            hour,
            min,
            action,
        })
    }
}

enum GuardState {
    Initial,
    Awake { guard: usize },
    Asleep { guard: usize, asleep_min: usize },
}

fn main() -> Result<()> {
    let mut sleep_by_guard = HashMap::new();
    let mut state = GuardState::Initial;
    let mut lines = io::stdin()
        .lock()
        .lines()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    lines.sort();
    for line in lines {
        let event: Event = line.parse()?;
        match event.action {
            Action::BeginShift { guard } => match state {
                GuardState::Initial | GuardState::Awake { .. } => {
                    state = GuardState::Awake { guard };
                }
                GuardState::Asleep { .. } => return Err("guard change while asleep".into()),
            },
            Action::Sleep => match state {
                GuardState::Awake { guard } => {
                    state = GuardState::Asleep {
                        guard,
                        asleep_min: event.min,
                    };
                }
                _ => return Err("no awake guard to sleep".into()),
            },
            Action::Wake => match state {
                GuardState::Asleep { guard, asleep_min } => {
                    let awake_min = event.min;
                    if awake_min < asleep_min {
                        return Err("out of order events".into());
                    }
                    let mins = sleep_by_guard
                        .entry(guard)
                        .or_insert_with(|| vec![0u32; 60]);
                    for i in asleep_min..awake_min {
                        mins[i] += 1;
                    }
                    state = GuardState::Awake { guard };
                }
                _ => return Err("no asleep guard to wake".into()),
            },
        }
    }

    println!("{}", part1(&sleep_by_guard)?);
    println!("{}", part2(&sleep_by_guard)?);
    Ok(())
}

fn part1(sleep_by_guard: &HashMap<usize, Vec<u32>>) -> Result<usize> {
    let (_, guard) = sleep_by_guard
        .iter()
        .map(|(guard, sleep)| (sleep.iter().sum::<u32>(), guard))
        .max()
        .ok_or_else(|| Box::<Error>::from("empty input"))?;
    let (_, min) = sleep_by_guard[guard]
        .iter()
        .enumerate()
        .map(|(min, count)| (count, min))
        .max()
        .expect("no sleep");
    Ok(guard * min)
}

fn part2(sleep_by_guard: &HashMap<usize, Vec<u32>>) -> Result<usize> {
    let (_, guard, min) = sleep_by_guard
        .iter()
        .map(|(guard, sleep)| {
            let (count, min) = sleep
                .iter()
                .enumerate()
                .map(|(min, count)| (count, min))
                .max()
                .expect("no sleep");
            (count, guard, min)
        })
        .max()
        .ok_or_else(|| Box::<Error>::from("empty input"))?;
    Ok(guard * min)
}
