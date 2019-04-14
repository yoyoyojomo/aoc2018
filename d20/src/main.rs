use std::collections::HashSet;
use std::io::{self, Read};
use std::mem;
use std::result;
use std::error::Error;

type Result<T> = result::Result<T, Box<Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coord(i32, i32);

#[derive(Debug, PartialEq, Eq, Hash)]
struct Door(Coord, Coord);

impl Door {
    fn new(a: Coord, b: Coord) -> Door {
        if a < b {
            Door(a, b)
        } else {
            Door(b, a)
        }
    }
}

struct Map {
    doors: HashSet<Door>,
}

impl Map {
    fn bfs(s: &[u8], mut i: usize, pos: &mut Vec<Coord>, doors: &mut HashSet<Door>) -> usize {
        let start_pos = pos.clone();
        let mut end_pos = Vec::new();
        while i < s.len() {
            let (offset_x, offset_y) = match s[i] {
                b'N' => (0, 1),
                b'E' => (1, 0),
                b'S' => (0, -1),
                b'W' => (-1, 0),
                b'(' => {
                    i = Map::bfs(s, i + 1, pos, doors);
                    continue;
                }
                b'|' => {
                    end_pos.extend_from_slice(&pos);
                    pos.clear();
                    pos.extend_from_slice(&start_pos);
                    i += 1;
                    continue;
                }
                b')' => {
                    mem::swap(pos, &mut end_pos);
                    pos.extend_from_slice(&end_pos);
                    pos.sort();
                    pos.dedup();
                    return i + 1;
                }
                b'$' => {
                    assert_eq!(&s[i..], b"$\n");
                    break;
                }
                _ => panic!("Unknown char"),
            };
            for pos in pos.iter_mut() {
                let Coord(x, y) = *pos;
                mem::replace(pos, Coord(x + offset_x, y + offset_y));
                doors.insert(Door::new(Coord(x, y), *pos));
            }
            i += 1;
        }
        s.len()
    }

    fn from_bytes(s: &[u8]) -> Result<Map> {
        assert_eq!(s[0], b'^');
        let mut doors = HashSet::new();
        let i = Map::bfs(s, 1, &mut vec![Coord(0, 0)], &mut doors);
        assert_eq!(i, s.len());
        Ok(Map { doors })
    }

    fn distances(&self) -> Vec<u32> {
        let mut distances = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![(0, Coord(0, 0))];
        while let Some((dist, Coord(x, y))) = stack.pop() {
            if !visited.insert(Coord(x, y)) {
                continue;
            }
            distances.push(dist);
            for (offset_x, offset_y) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let neighbor = Coord(x + offset_x, y + offset_y);
                if Coord(x, y) < neighbor && self.doors.contains(&Door(Coord(x, y), neighbor)) || self.doors.contains(&Door(neighbor, Coord(x, y))) {
                    stack.push((dist + 1, neighbor));
                }
            }
        }
        distances
    }

    fn furthest_room(&self) -> u32 {
        self.distances().into_iter().max().unwrap()
    }
}

fn main() -> Result<()> {
    assert_eq!(Map::from_bytes(b"^WNE$\n")?.furthest_room(), 3);
    assert_eq!(Map::from_bytes(b"^ENWWW(NEEE|SSE(EE|N))$\n")?.furthest_room(), 10);
    assert_eq!(Map::from_bytes(b"^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$\n")?.furthest_room(), 18);
    assert_eq!(Map::from_bytes(b"^(N|S)(E|W)$\n")?.doors.len(), 6);

    let bytes: Vec<u8> = io::stdin().bytes().collect::<result::Result<_, _>>()?;
    let map = Map::from_bytes(&bytes)?;
    println!("{}", map.furthest_room());
    println!("{}", map.distances().into_iter().filter(|&d| d >= 1000).count());
    Ok(())
}
