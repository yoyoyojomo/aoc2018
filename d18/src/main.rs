use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::mem;
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Open,
    Tree,
    Lumber,
}

struct Area {
    width: usize,
    tiles: Vec<Tile>,
    scratch: Vec<Tile>,
    time: usize,
    history: HashMap<u64, usize>,
    periodicity: Option<usize>,
}

impl Area {
    fn from_bytes(bytes: &mut impl Iterator<Item = io::Result<u8>>) -> Result<Self> {
        let mut width = None;
        let mut tiles = Vec::new();
        while let Some(c) = bytes.next() {
            match c? {
                b'.' => tiles.push(Tile::Open),
                b'|' => tiles.push(Tile::Tree),
                b'#' => tiles.push(Tile::Lumber),
                b'\n' => {
                    if let Some(width) = width {
                        if tiles.len() % width != 0 {
                            return Err("parse failed".into());
                        }
                    } else {
                        width = Some(tiles.len());
                    }
                }
                _ => return Err("parse failed".into()),
            }
        }
        match width {
            Some(width) if tiles.len() % width == 0 => {
                let scratch = vec![Tile::Open; tiles.len()];
                Ok(Area {
                    width,
                    tiles,
                    scratch,
                    time: 0,
                    history: HashMap::new(),
                    periodicity: None,
                })
            }
            _ => Err("parse failed".into()),
        }
    }

    fn adjacencies(&self, i: usize) -> (usize, usize, usize) {
        let offsets = [self.width - 1, self.width, 1, self.width + 1];
        let (mut open, mut tree, mut lumber) = (0, 0, 0);
        // assumes width > 1
        let (neg_offsets, pos_offsets) = match i % self.width {
            0 => (&offsets[0..2], &offsets[1..4]),
            x if x == self.width - 1 => (&offsets[1..4], &offsets[0..2]),
            _ => (&offsets[..], &offsets[..]),
        };
        for &offset in neg_offsets {
            if i >= offset {
                match self.tiles[i - offset] {
                    Tile::Open => open += 1,
                    Tile::Tree => tree += 1,
                    Tile::Lumber => lumber += 1,
                }
            }
        }
        for &offset in pos_offsets {
            if i + offset < self.tiles.len() {
                match self.tiles[i + offset] {
                    Tile::Open => open += 1,
                    Tile::Tree => tree += 1,
                    Tile::Lumber => lumber += 1,
                }
            }
        }
        (open, tree, lumber)
    }

    fn step(&mut self) {
        for i in 0..self.tiles.len() {
            let (_open, tree, lumber) = self.adjacencies(i);
            let tile = match self.tiles[i] {
                Tile::Open => {
                    if tree >= 3 {
                        Tile::Tree
                    } else {
                        Tile::Open
                    }
                }
                Tile::Tree => {
                    if lumber >= 3 {
                        Tile::Lumber
                    } else {
                        Tile::Tree
                    }
                }
                Tile::Lumber => {
                    if lumber >= 1 && tree >= 1 {
                        Tile::Lumber
                    } else {
                        Tile::Open
                    }
                }
            };
            self.scratch[i] = tile;
        }
        mem::swap(&mut self.tiles, &mut self.scratch);
        self.time += 1;

        if self.periodicity.is_none() {
            let mut hasher = DefaultHasher::new();
            self.tiles.hash(&mut hasher);
            let hash = hasher.finish();
            if let Some(prev) = self.history.insert(hash, self.time) {
                self.periodicity = Some(self.time - prev);
            }
        }
    }

    fn trees(&self) -> usize {
        self.tiles.iter().filter(|&&t| t == Tile::Tree).count()
    }

    fn lumbers(&self) -> usize {
        self.tiles.iter().filter(|&&t| t == Tile::Lumber).count()
    }

    fn periodicity(&self) -> Option<usize> {
        self.periodicity
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, t) in self.tiles.iter().enumerate() {
            let c = match t {
                Tile::Open => '.',
                Tile::Tree => '|',
                Tile::Lumber => '#',
            };
            write!(f, "{}", c)?;
            if i % self.width == self.width - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut area = Area::from_bytes(&mut io::stdin().bytes())?;
    for _ in 0..10 {
        area.step();
    }
    println!("{}", area.trees() * area.lumbers());

    let mut i = 10;
    while i < 1000000000 {
        area.step();
        i += 1;
        if let Some(p) = area.periodicity() {
            i += ((1000000000 - i) / p) * p;
        }
    }
    println!("{}", area.trees() * area.lumbers());
    Ok(())
}
