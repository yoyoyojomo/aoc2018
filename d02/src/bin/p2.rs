use std::collections::HashSet;
use std::io::{self, BufRead};

fn main() {
    let mut seen_at = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap().into_bytes();
        for i in 0..line.len() {
            if i == seen_at.len() {
                seen_at.push(HashSet::new());
            }
            let seen = &mut seen_at[i];
            let mut spliced = line.clone();
            spliced.remove(i);
            if !seen.insert(spliced.clone()) {
                println!("{}", String::from_utf8(spliced).unwrap());
                return;
            }
        }
    }
}
