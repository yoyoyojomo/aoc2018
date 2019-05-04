use std::cmp;
use std::collections::BinaryHeap;
use std::error::Error;
use std::io::{self, Read};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl Point {
    fn new(x: i32, y: i32, z: i32) -> Point {
        Point { x, y, z }
    }
}

struct Nanobot {
    pos: Point,
    r: i32,
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

fn parse_i32_until(bytes: &mut impl Iterator<Item = io::Result<u8>>, until: u8) -> Result<i32> {
    let mut num = 0;
    let mut mult = 1;
    loop {
        match bytes.next() {
            Some(Ok(b)) if b == until => break,
            Some(Ok(b)) if b == b'-' => mult = -1, // should only match beginning
            Some(Ok(b)) if b >= b'0' && b <= b'9' => num = num * 10 + (b - b'0') as i32,
            _ => return Err("parse failed".into()),
        }
    }
    Ok(mult * num)
}

impl Nanobot {
    fn from_bytes(
        bytes: &mut impl Iterator<Item = result::Result<u8, io::Error>>,
    ) -> Result<Nanobot> {
        consume_bytes(bytes, b"pos=<")?;
        let x = parse_i32_until(bytes, b',')?;
        let y = parse_i32_until(bytes, b',')?;
        let z = parse_i32_until(bytes, b'>')?;
        consume_bytes(bytes, b", r=")?;
        let r = parse_i32_until(bytes, b'\n')?;
        Ok(Nanobot {
            pos: Point { x, y, z },
            r,
        })
    }

    fn distance_to(&self, o: &Nanobot) -> i32 {
        (self.pos.x - o.pos.x).abs() + (self.pos.y - o.pos.y).abs() + (self.pos.z - o.pos.z).abs()
    }
}

#[derive(PartialEq, Eq)]
struct Subdivision {
    l: Point,
    d: i32,
    bots: Vec<usize>,
}

impl Ord for Subdivision {
    fn cmp(&self, other: &Subdivision) -> cmp::Ordering {
        (self.bots.len(), other.distance_to_origin(), other.d).cmp(&(
            other.bots.len(),
            self.distance_to_origin(),
            self.d,
        ))
    }
}

impl PartialOrd for Subdivision {
    fn partial_cmp(&self, other: &Subdivision) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn interval_distance(v: i32, l: i32, u: i32) -> i32 {
    match (v < l, v > u) {
        (true, false) => l - v,
        (false, true) => v - u,
        (false, false) => 0,
        (true, true) => unreachable!(),
    }
}

impl Subdivision {
    fn world(bots: &[Nanobot]) -> Subdivision {
        let v = -1 << 29;
        let ret = Subdivision {
            l: Point::new(v, v, v),
            d: 1 << 30,
            bots: (0..bots.len()).collect(),
        };
        for b in bots {
            assert!(ret.intersects(b));
        }
        ret
    }

    fn l(&self) -> Point {
        self.l
    }

    fn u(&self) -> Point {
        let Point { x, y, z } = self.l;
        Point::new(x + self.d - 1, y + self.d - 1, z + self.d - 1)
    }

    fn intersects(&self, b: &Nanobot) -> bool {
        (interval_distance(b.pos.x, self.l().x, self.u().x)
            + interval_distance(b.pos.y, self.l().y, self.u().y)
            + interval_distance(b.pos.z, self.l().z, self.u().z))
            <= b.r
    }

    fn distance_to_origin(&self) -> i32 {
        cmp::min(self.l().x.abs(), self.u().x.abs())
            + cmp::min(self.l().y.abs(), self.u().y.abs())
            + cmp::min(self.l().z.abs(), self.u().z.abs())
    }

    fn split(&self) -> [Subdivision; 8] {
        let Point { x, y, z } = self.l;
        assert!(self.d > 1);
        let d = self.d / 2;
        [
            Subdivision {
                l: self.l,
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x + d, y, z),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x + d, y + d, z),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x + d, y + d, z + d),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x + d, y, z + d),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x, y + d, z),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x, y + d, z + d),
                d,
                bots: Vec::new(),
            },
            Subdivision {
                l: Point::new(x, y, z + d),
                d,
                bots: Vec::new(),
            },
        ]
    }
}

fn main() -> Result<()> {
    let mut bytes = io::stdin().bytes().peekable();
    let mut bots = Vec::new();
    while bytes.peek().is_some() {
        bots.push(Nanobot::from_bytes(&mut bytes)?);
    }
    let strongest = bots
        .iter()
        .max_by_key(|x| x.r)
        .ok_or_else(|| Box::<Error>::from("empty"))?;

    println!(
        "{}",
        bots.iter()
            .filter(|x| strongest.distance_to(x) <= strongest.r)
            .count()
    );

    let root = Subdivision::world(&bots);
    let mut pq = BinaryHeap::new();
    pq.push(root);
    while let Some(node) = pq.pop() {
        if node.d == 1 {
            println!("{}", node.distance_to_origin());
            break;
        }
        let children = node.split();
        for mut child in Vec::from(Box::new(children) as Box<[Subdivision]>) {
            for &i in &node.bots {
                if child.intersects(&bots[i]) {
                    child.bots.push(i);
                }
            }
            pq.push(child);
        }
    }

    Ok(())
}
