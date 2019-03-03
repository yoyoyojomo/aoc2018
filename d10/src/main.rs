use std::io::{self, BufRead};
use std::str::{self, FromStr};
use std::result;
use std::error::Error;
use std::iter::Peekable;

type Result<T> = result::Result<T, Box<Error>>;

fn consume_str<T: Iterator<Item = u8>>(it: &mut T, s: &[u8]) -> Result<()> {
    for &c in s {
        match it.next() {
            Some(x) if x == c => {},
            _ => return Err("parse failed".into()),
        }
    }
    Ok(())
}

fn parse_i32<T: Iterator<Item = u8>>(it: &mut Peekable<T>) -> Result<i32> {
    let mut num = Vec::new();
    while let Some(&c) = it.peek() {
        if c == b' ' {
            // noop
        } else if c == b'-' || c.is_ascii_digit() {
            num.push(c);
        } else {
            break;
        }
        it.next();
    }
    unsafe {
        Ok(str::from_utf8_unchecked(&num).parse()?)
    }
}

struct Star {
    initial: (i32, i32),
    velocity: (i32, i32),
}

impl Star {
    fn at(&self, t: i32) -> (i32, i32) {
        let (ix, iy) = self.initial;
        let (vx, vy) = self.velocity;
        (ix + t * vx, iy + t * vy)
    }
}

impl FromStr for Star {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Star> {
        let mut it = s.bytes().peekable();
        consume_str(&mut it, b"position=<")?;
        let ix = parse_i32(&mut it)?;
        consume_str(&mut it, b",")?;
        let iy = parse_i32(&mut it)?;
        consume_str(&mut it, b"> velocity=<")?;
        let vx = parse_i32(&mut it)?;
        consume_str(&mut it, b",")?;
        let vy = parse_i32(&mut it)?;
        consume_str(&mut it, b">")?;
        if it.peek() != None {
            return Err("trailing input".into());
        }
        Ok(Star { initial: (ix, iy), velocity: (vx, vy) })
    }
}

fn bounds_of(pos: &Vec<(i32, i32)>) -> (i32, i32, i32, i32) {
    let xmin = pos.iter().map(|&(x, _)| x).min().unwrap();
    let xmax = pos.iter().map(|&(x, _)| x).max().unwrap();
    let ymin = pos.iter().map(|&(_, y)| y).min().unwrap();
    let ymax = pos.iter().map(|&(_, y)| y).max().unwrap();
    (xmin, ymin, xmax, ymax)
}

struct Constellation {
    stars: Vec<Star>,
}

impl Constellation {
    fn new() -> Self {
        Self { stars: Vec::new() }
    }

    fn push(&mut self, star: Star) {
        self.stars.push(star);
    }

    fn linear_size(&self, t: i32) -> i32 {
        let pos: Vec<_> = self.stars.iter().map(|s| s.at(t)).collect();
        let (xmin, ymin, xmax, ymax) = bounds_of(&pos);
        (xmax - xmin) + (ymax - ymin)
    }
}

fn main() -> Result<()> {
    let mut constellation = Constellation::new();
    for line in io::stdin().lock().lines() {
        constellation.push(line?.parse()?);
    }

    // Binary search for smallest bounding box.
    let (mut tmin, mut tmax) = (0, 1 << 20);
    while tmin != tmax {
        let tmid = (tmax + tmin) / 2;
        if constellation.linear_size(tmid) > constellation.linear_size(tmid + 1) {
            tmin = tmid + 1;
        } else {
            tmax = tmid;
        }
    }

    let pos: Vec<_> = constellation.stars.iter().map(|s| s.at(tmin)).collect();
    let (xmin, ymin, xmax, ymax) = bounds_of(&pos);
    for y in ymin..=ymax {
        for x in xmin..=xmax {
            if pos.contains(&(x, y)) {
                print!("*");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!("{}", tmin);

    Ok(())
}
