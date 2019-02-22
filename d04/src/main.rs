use std::io::{self, BufRead};
use std::iter::Peekable;
use std::str::{Chars, FromStr};

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

// TODO try as a newtype?
struct StrParser<'a> {
    iter: Peekable<Chars<'a>>,
}

impl<'a> StrParser<'a> {
    fn new(s: &'a str) -> Self {
        Self { iter: s.chars().peekable() }
    }

    fn done(&mut self) -> bool {
        // TODO more succinct way to write this?
        match self.iter.peek() {
            Some(_) => false,
            None => true,
        }
    }

    // TODO for this to work predictably, nothing should be consumed on error
    fn consume_str(&mut self, s: &str) -> Result<()> {
        for c in s.chars() {
            if self.iter.next() != Some(c) {
                return Err("unexpected parse".into());
            }
        }
        Ok(())
    }

    fn parse_usize(&mut self) -> Result<usize> {
        let mut digits = String::new();
        while let Some(&c) = self.iter.peek() {
            if !c.is_numeric() {
                break;
            }
            digits.push(c);
            self.iter.next();
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
    type Err = Box<::std::error::Error>;

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
        // TODO weird "f" truncation because of consume_str issue
        } else if let Ok(()) = parser.consume_str("alls asleep") {
            Action::Sleep
        // TODO ditto "wak"
        } else if let Ok(()) = parser.consume_str("es up") {
            Action::Wake
        } else {
            return Err("unexpected action".into());
        };
        if !parser.done() {
            return Err("trailing input".into());
        }
        Ok(Event {
            year, month, day, hour, min, action,
        })
    }
}

fn main() -> Result<()> {
    for line in io::stdin().lock().lines() {
        let event: Event = line?.parse()?;
        dbg!(event);
    }
    Ok(())
}
