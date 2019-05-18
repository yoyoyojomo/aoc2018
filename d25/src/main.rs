use failure::{self, bail};
use std::io::{self, BufRead};
use std::result;
use std::str::FromStr;

type Result<T> = result::Result<T, failure::Error>;

struct Coord(i32, i32, i32, i32);

impl Coord {
    fn distance(&self, o: &Coord) -> i32 {
        (self.0 - o.0).abs() + (self.1 - o.1).abs() + (self.2 - o.2).abs() + (self.3 - o.3).abs()
    }
}

impl FromStr for Coord {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Coord> {
        let vals: Vec<i32> = s
            .split(',')
            .map(str::parse)
            .collect::<result::Result<_, _>>()?;
        match vals.as_slice() {
            &[a, b, c, d] => Ok(Coord(a, b, c, d)),
            _ => bail!("parse error"),
        }
    }
}

struct UnionFind {
    coords: Vec<Coord>,
    parents: Vec<usize>,
}

impl UnionFind {
    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parents[rb] = ra;
        }
    }

    fn find(&self, mut a: usize) -> usize {
        while a != self.parents[a] {
            a = self.parents[a];
        }
        return a;
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    // why does this compile?
    let coords: Vec<Coord> = stdin
        .lock()
        .lines()
        .flat_map(|l| l.map_err(|e| e.into()).and_then(|s| s.parse()))
        .collect();
    // let mut coords: Vec<Coord> = Vec::new();
    // for line in stdin.lock().lines() {
    //     coords.push(line?.parse()?);
    // }

    let parents = (0..coords.len()).collect();
    let mut uf = UnionFind { coords, parents };
    for i in 0..uf.coords.len() {
        for j in i + 1..uf.coords.len() {
            if uf.coords[i].distance(&uf.coords[j]) <= 3 {
                uf.union(i, j);
            }
        }
    }

    let mut roots: Vec<usize> = (0..uf.coords.len()).map(|i| uf.find(i)).collect();
    roots.sort();
    roots.dedup();
    println!("{}", roots.len());

    Ok(())
}
