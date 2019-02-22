use std::io::{self, BufRead};

fn repeated_char(s: String) -> (bool, bool) {
    let mut has_2 = false;
    let mut has_3 = false;
    let mut s = s.into_bytes();
    s.sort();
    let mut iter = s.iter().peekable();
    while let Some(ch) = iter.next() {
        let mut count = 1;
        while iter.peek() == Some(&&ch) {
            count += 1;
            iter.next();
        }
        if count == 2 {
            has_2 = true;
        }
        if count == 3 {
            has_3 = true;
        }
    }
    (has_2, has_3)
}

fn main() {
    let stdin = io::stdin();
    let (has_2s, has_3s): (Vec<_>, Vec<_>) = stdin
        .lock()
        .lines()
        .map(|line| repeated_char(line.unwrap()))
        .unzip();
    let checksum = has_2s.iter().filter(|&&x| x).count() * has_3s.iter().filter(|&&x| x).count();
    println!("{}", checksum);
}
