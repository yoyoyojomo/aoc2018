use std::io::{self, BufRead};
use std::iter::Peekable;

struct Claim {
    id: usize,
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}

fn consume_str<T>(iter: &mut T, s: &str)
where
    T: Iterator<Item = char>,
{
    for c in s.chars() {
        if iter.next() != Some(c) {
            panic!("malformed");
        }
    }
}

fn parse_usize<T>(iter: &mut Peekable<T>) -> usize
where
    T: Iterator<Item = char>,
{
    let mut digits = String::new();
    while let Some(&ch) = iter.peek() {
        if !ch.is_numeric() {
            break;
        }
        digits.push(ch);
        iter.next();
    }
    digits.parse().unwrap()
}

fn main() {
    let claims = io::stdin()
        .lock()
        .lines()
        .map(|line| {
            let line = line.unwrap();
            let mut iter = line.chars().peekable();
            consume_str(&mut iter, "#");
            let id = parse_usize(&mut iter);
            consume_str(&mut iter, " @ ");
            let left = parse_usize(&mut iter);
            consume_str(&mut iter, ",");
            let top = parse_usize(&mut iter);
            consume_str(&mut iter, ": ");
            let width = parse_usize(&mut iter);
            consume_str(&mut iter, "x");
            let height = parse_usize(&mut iter);
            if iter.next() != None {
                panic!("unexpected chars");
            }
            Claim {
                id,
                left,
                top,
                width,
                height,
            }
        })
        .collect::<Vec<_>>();

    let y_max = claims.iter().map(|c| c.top + c.height).max().unwrap();
    let x_max = claims.iter().map(|c| c.left + c.width).max().unwrap();
    let mut overlaps = vec![0; y_max * x_max];

    for claim in claims {
        for y in claim.top..claim.top + claim.height {
            for x in claim.left..claim.left + claim.width {
                overlaps[y * x_max + x] += 1;
            }
        }
    }

    println!("{}", overlaps.iter().filter(|&&c| c > 1).count());
}
