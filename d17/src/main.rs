use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::result;
use std::usize;

type Result<T> = result::Result<T, Box<Error>>;

struct Vein {
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
}

fn consume_bytes(bytes: &mut impl Iterator<Item = io::Result<u8>>, s: &[u8]) -> Result<()> {
    for b in s {
        match bytes.next() {
            Some(Ok(c)) if c == *b => {}
            _ => return Err("parse failed".into()),
        }
    }
    Ok(())
}

fn parse_usize_until(bytes: &mut impl Iterator<Item = io::Result<u8>>, until: u8) -> Result<usize> {
    let mut num = 0;
    loop {
        match bytes.next() {
            Some(Ok(b)) if b == until => break,
            Some(Ok(b)) if b >= b'0' && b <= b'9' => num = num * 10 + (b - b'0') as usize,
            _ => return Err("parse failed".into()),
        }
    }
    Ok(num)
}

impl Vein {
    fn from_bytes(bytes: &mut impl Iterator<Item = io::Result<u8>>) -> Result<Vein> {
        let xfirst = match bytes.next() {
            Some(Ok(b'x')) => true,
            Some(Ok(b'y')) => false,
            _ => return Err("parse failed".into()),
        };
        consume_bytes(bytes, b"=")?;
        let first = parse_usize_until(bytes, b',')?;
        consume_bytes(bytes, b" ")?;
        let xsecond = match bytes.next() {
            Some(Ok(b'x')) => true,
            Some(Ok(b'y')) => false,
            _ => return Err("parse failed".into()),
        };
        assert_ne!(xfirst, xsecond);
        consume_bytes(bytes, b"=")?;
        let secondmin = parse_usize_until(bytes, b'.')?;
        consume_bytes(bytes, b".")?;
        let secondmax = parse_usize_until(bytes, b'\n')?;
        if xfirst {
            Ok(Vein {
                xmin: first,
                xmax: first,
                ymin: secondmin,
                ymax: secondmax,
            })
        } else {
            Ok(Vein {
                xmin: secondmin,
                xmax: secondmax,
                ymin: first,
                ymax: first,
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Coord(usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Clay,
    Settled,
    Passed,
}

struct World {
    tiles: HashMap<Coord, Tile>,
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.tiles.is_empty() {
            return Ok(());
        }
        for y in self.ymin..=self.ymax {
            for x in self.xmin..=self.xmax {
                let c = match self.tiles.get(&Coord(x, y)) {
                    Some(Tile::Clay) => '#',
                    Some(Tile::Settled) => '~',
                    Some(Tile::Passed) => '|',
                    None => '.',
                };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl World {
    fn new() -> Self {
        World {
            tiles: HashMap::new(),
            xmin: usize::MAX,
            xmax: usize::MIN,
            ymin: usize::MAX,
            ymax: usize::MIN,
        }
    }

    fn flows(&mut self, coord: Coord) -> bool {
        match self.tiles.get(&coord) {
            Some(Tile::Passed) => true,
            None => {
                self.set(coord, Tile::Passed);
                true
            }
            Some(Tile::Clay) | Some(Tile::Settled) => false,
        }
    }

    fn set(&mut self, coord: Coord, tile: Tile) {
        if coord.0 < self.xmin {
            self.xmin = coord.0;
        }
        if coord.0 > self.xmax {
            self.xmax = coord.0;
        }
        if coord.1 < self.ymin {
            self.ymin = coord.1;
        }
        if coord.1 > self.ymax {
            self.ymax = coord.1;
        }
        self.tiles.insert(coord, tile);
    }

    fn spill(&mut self, from: Coord, visited: &mut HashSet<Coord>) {
        if !visited.insert(from) {
            return;
        }
        let Coord(x, mut y) = from;
        if y < self.ymin {
            y = self.ymin;
        }
        self.set(Coord(x, y), Tile::Passed);
        // flow down
        while self.flows(Coord(x, y + 1)) {
            y += 1;
            if y >= self.ymax {
                return;
            }
        }
        // flow back
        while y >= self.ymin && !self.flows(Coord(x, y + 1)) {
            let mut bounded = true;
            // flow left
            let mut xleft = x;
            while self.flows(Coord(xleft - 1, y)) {
                xleft -= 1;
                let below = Coord(xleft, y + 1);
                if self.flows(below) {
                    self.spill(below, visited);
                }
                if self.flows(below) {
                    bounded = false;
                    break;
                }
            }
            // flow right
            let mut xright = x;
            while self.flows(Coord(xright + 1, y)) {
                xright += 1;
                let below = Coord(xright, y + 1);
                if self.flows(below) {
                    self.spill(below, visited);
                }
                if self.flows(below) {
                    bounded = false;
                    break;
                }
            }
            if bounded {
                for x in xleft..=xright {
                    self.set(Coord(x, y), Tile::Settled);
                }
            }
            y -= 1;
        }
    }

    fn count_reachable(&self) -> usize {
        self.tiles.values().filter(|&&t| t == Tile::Settled || t == Tile::Passed).count()
    }

    fn count_settled(&self) -> usize {
        self.tiles.values().filter(|&&t| t == Tile::Settled).count()
    }
}

fn main() -> Result<()> {
    let mut bytes = io::stdin().bytes().peekable();
    let mut world = World::new();
    while let Some(_) = bytes.peek() {
        let vein = Vein::from_bytes(&mut bytes)?;
        for x in vein.xmin..=vein.xmax {
            for y in vein.ymin..=vein.ymax {
                world.set(Coord(x, y), Tile::Clay);
            }
        }
    }

    world.spill(Coord(500, 0), &mut HashSet::new());
    println!("{}", world.count_reachable());
    println!("{}", world.count_settled());

    Ok(())
}
