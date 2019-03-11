use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::mem;
use std::result;
use std::usize;

type Result<T> = result::Result<T, Box<Error>>;

enum Track {
    Empty,
    Vertical,
    Horizontal,
    Intersection,
    CurveSlash,
    CurveBackslash,
}

#[derive(Clone, Copy)]
enum Direction {
    N,
    E,
    W,
    S,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinates(usize, usize);

impl PartialOrd for Coordinates {
    fn partial_cmp(&self, other: &Coordinates) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Coordinates {
    fn cmp(&self, other: &Coordinates) -> Ordering {
        (self.1, self.0).cmp(&(other.1, other.0))
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl Coordinates {
    fn shift(&mut self, direction: Direction) {
        match direction {
            Direction::N => self.1 -= 1,
            Direction::E => self.0 += 1,
            Direction::W => self.0 -= 1,
            Direction::S => self.1 += 1,
        }
    }
}

#[derive(Clone, Copy)]
enum OnIntersection {
    Left,
    Straight,
    Right,
}

struct Cart {
    position: Coordinates,
    direction: Direction,
    on_intersection: OnIntersection,
}

impl Cart {
    fn move_on_track(&mut self, track: &Track) -> Result<()> {
        use Direction::*;
        self.direction = match track {
            Track::Empty => return Err("cart off track".into()),
            Track::Vertical => match self.direction {
                N | S => self.direction,
                _ => return Err("horizontal cart on vertical track".into()),
            },
            Track::Horizontal => match self.direction {
                E | W => self.direction,
                _ => return Err("vertical cart on horizontal track".into()),
            },
            Track::Intersection => match self.on_intersection {
                OnIntersection::Left => {
                    self.on_intersection = OnIntersection::Straight;
                    match self.direction {
                        N => W,
                        E => N,
                        W => S,
                        S => E,
                    }
                }
                OnIntersection::Straight => {
                    self.on_intersection = OnIntersection::Right;
                    self.direction
                }
                OnIntersection::Right => {
                    self.on_intersection = OnIntersection::Left;
                    match self.direction {
                        N => E,
                        E => S,
                        W => N,
                        S => W,
                    }
                }
            },
            Track::CurveSlash => match self.direction {
                N => E,
                E => N,
                W => S,
                S => W,
            },
            Track::CurveBackslash => match self.direction {
                N => W,
                E => S,
                W => N,
                S => E,
            },
        };
        self.position.shift(self.direction);
        Ok(())
    }
}

struct Map {
    width: usize,
    tracks: Vec<Track>,
    carts: Vec<Cart>,
}

fn create_cart(byte_pos: usize, width: usize, direction: Direction) -> Cart {
    let position = Coordinates(byte_pos % width, byte_pos / width);
    Cart {
        position,
        direction,
        on_intersection: OnIntersection::Left,
    }
}

impl Map {
    fn from_bytes<T, E>(bytes: T) -> Result<Map>
    where
        T: Iterator<Item = result::Result<u8, E>>,
        E: Error + 'static,
    {
        let mut width = usize::MAX;
        let mut tracks = Vec::new();
        let mut carts = Vec::new();
        for byte in bytes {
            let byte = byte?;
            match byte {
                b' ' => tracks.push(Track::Empty),
                b'|' => tracks.push(Track::Vertical),
                b'-' => tracks.push(Track::Horizontal),
                b'/' => tracks.push(Track::CurveSlash),
                b'\\' => tracks.push(Track::CurveBackslash),
                b'+' => tracks.push(Track::Intersection),
                b'^' => {
                    carts.push(create_cart(tracks.len(), width, Direction::N));
                    tracks.push(Track::Vertical);
                }
                b'v' => {
                    carts.push(create_cart(tracks.len(), width, Direction::S));
                    tracks.push(Track::Vertical);
                }
                b'<' => {
                    carts.push(create_cart(tracks.len(), width, Direction::W));
                    tracks.push(Track::Horizontal);
                }
                b'>' => {
                    carts.push(create_cart(tracks.len(), width, Direction::E));
                    tracks.push(Track::Horizontal);
                }
                b'\n' => {
                    if width == usize::MAX {
                        width = tracks.len();
                    } else {
                        if tracks.len() % width != 0 {
                            return Err("uneven grid".into());
                        }
                    }
                }
                _ => return Err("invalid input".into()),
            }
        }
        Ok(Map {
            width,
            tracks,
            carts,
        })
    }

    fn tick(&mut self) -> Result<Vec<Coordinates>> {
        let mut crashes = Vec::new();
        let mut positions: HashSet<_> = self.carts.iter().map(|c| c.position).collect();
        let mut old_carts = Vec::new();
        mem::swap(&mut self.carts, &mut old_carts);
        for mut cart in old_carts {
            if crashes.contains(&cart.position) {
                continue;
            }
            let Coordinates(x, y) = cart.position;
            cart.move_on_track(&self.tracks[x + y * self.width])?;
            if positions.contains(&cart.position) {
                crashes.push(cart.position);
                self.carts.retain(|c| c.position != cart.position);
            } else {
                positions.insert(cart.position);
                self.carts.push(cart);
            }
            positions.remove(&Coordinates(x, y));
        }
        self.carts.sort_by_key(|c| c.position);
        Ok(crashes)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        for (i, track) in self.tracks.iter().enumerate() {
            let mut ch = match track {
                Track::Empty => ' ',
                Track::Vertical => '|',
                Track::Horizontal => '-',
                Track::Intersection => '+',
                Track::CurveSlash => '/',
                Track::CurveBackslash => '\\',
            };
            let position = Coordinates(i % self.width, i / self.width);
            let carts: Vec<_> = self
                .carts
                .iter()
                .filter(|c| c.position == position)
                .collect();
            if carts.len() > 1 {
                ch = 'X';
            } else if carts.len() == 1 {
                ch = match carts[0].direction {
                    Direction::N => '^',
                    Direction::E => '>',
                    Direction::W => '<',
                    Direction::S => 'v',
                }
            }
            write!(f, "{}", ch)?;
            if (i + 1) % self.width == 0 {
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut map = Map::from_bytes(io::stdin().bytes())?;
    let mut has_crash = false;
    while map.carts.len() > 1 {
        let crashes = map.tick()?;
        if !has_crash && !crashes.is_empty() {
            println!("{}", crashes[0]);
            has_crash = true;
        }
    }
    if map.carts.is_empty() {
        return Err("no remaining carts".into());
    }
    println!("{}", map.carts[0].position);
    Ok(())
}
