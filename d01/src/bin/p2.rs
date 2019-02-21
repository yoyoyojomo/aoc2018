use std::collections::HashSet;
use std::io::{self, BufRead};

fn main() {
    let numbers = io::stdin()
        .lock()
        .lines()
        .map(|x| x.unwrap().parse().unwrap())
        .collect::<Vec<_>>();
    let mut seen = HashSet::new();
    let mut sum = 0;
    for num in numbers.iter().cycle() {
        if !seen.insert(sum) {
            break;
        }
        sum += num;
    }
    println!("{}", sum);
}
