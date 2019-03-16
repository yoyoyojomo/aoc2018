use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::result;
use std::usize;

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(PartialEq)]
struct UnitId(usize);

#[derive(PartialEq)]
enum Tile {
    Wall,
    Open,
    Unit(UnitId),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum UnitKind {
    Goblin,
    Elf,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Unit {
    hp: i16,
    pos: usize,
    kind: UnitKind,
}

struct Board {
    tiles: Vec<Tile>,
    units: Vec<Unit>,
    width: usize,
}

impl Board {
    fn from_bytes(bytes: io::Bytes<io::Stdin>) -> Result<Board> {
        let mut tiles = Vec::new();
        let mut units = Vec::new();
        let mut width = 0;
        for byte in bytes {
            let byte = byte?;
            if byte == b'\n' {
                if width == 0 {
                    width = tiles.len();
                } else if tiles.len() % width != 0 {
                    return Err("non-rectangular input".into());
                }
                continue;
            }
            let tile = match byte {
                b'#' => Tile::Wall,
                b'.' => Tile::Open,
                b'E' | b'G' => {
                    let unit_id = UnitId(units.len());
                    units.push(Unit {
                        kind: match byte {
                            b'E' => UnitKind::Elf,
                            b'G' | _ => UnitKind::Goblin,
                        },
                        hp: 200,
                        pos: tiles.len(),
                    });
                    Tile::Unit(unit_id)
                }
                _ => return Err("invalid byte".into()),
            };
            tiles.push(tile);
        }
        Ok(Board {
            tiles,
            units,
            width,
        })
    }

    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> {
        // Assumes board bordered by walls.
        vec![pos - self.width, pos - 1, pos + 1, pos + self.width]
            .into_iter()
    }

    fn open_neighbors(&self, pos: usize) -> impl Iterator<Item = usize> {
        self.neighbors(pos)
            .filter(|&pos| self.tiles[pos] == Tile::Open)
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn foe_neighbors(&self, pos: usize, kind: UnitKind) -> impl Iterator<Item = UnitId> {
        self.neighbors(pos).filter_map(|pos| match self.tiles[pos] {
            Tile::Unit(UnitId(unit_id)) => {
                if self.units[unit_id].kind != kind {
                    Some(UnitId(unit_id))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .into_iter()
    }

    fn bfs_step(&self, src: usize, dst: Vec<usize>) -> Option<usize> {
        let mut dst_idx = vec![false; self.tiles.len()];
        for &d in &dst {
            dst_idx[d] = true;
        }
        let mut distances = vec![usize::MAX; self.tiles.len()];
        distances[src] = 0;
        let mut max_distance = usize::MAX;
        let mut fifo = VecDeque::new();
        fifo.push_back((0, src));
        while let Some((distance, pos)) = fifo.pop_front() {
            if distance > max_distance {
                break;
            }
            if distance > distances[pos] {
                continue;
            } else {
                distances[pos] = distance;
            }
            if dst_idx[pos] {
                max_distance = distance;
            }
            for neighbor in self.open_neighbors(pos) {
                fifo.push_back((distance + 1, neighbor));
            }
        }

        let mut positions: Vec<_> = dst
            .into_iter()
            .filter(|&d| distances[d] == max_distance)
            .collect();
        let mut distance = max_distance;
        if distance == usize::MAX {
            return None;
        }
        while distance > 1 {
            distance -= 1;
            positions = positions
                .into_iter()
                .flat_map(|p| self.open_neighbors(p))
                .filter(|&p| distances[p] == distance)
                .collect();
            positions.sort();
            positions.dedup();
        }
        Some(positions[0])
    }

    fn unit_move(&mut self, unit_id: UnitId) -> bool {
        let UnitId(unit_id) = unit_id;
        let unit = &self.units[unit_id];
        let targets: Vec<_> = self
            .units
            .iter()
            .enumerate()
            .filter(|(i, target)| {
                *i != unit_id && target.hp > 0 && target.kind != unit.kind
            })
            .map(|(i, _)| UnitId(i))
            .collect();
        if targets.is_empty() {
            return false;
        }
        let targets: Vec<_> = targets
            .iter()
            .flat_map(|&UnitId(id)| self.open_neighbors(self.units[id].pos))
            .collect();
        if let Some(next_pos) = self.bfs_step(unit.pos, targets) {
            self.tiles.swap(unit.pos, next_pos);
            self.units[unit_id].pos = next_pos;
        }
        true
    }

    fn next_round(&mut self) -> bool {
        let mut unit_ids: Vec<UnitId> = (0..self.units.len())
            .filter(|&id| self.units[id].hp > 0)
            .map(|id| UnitId(id))
            .collect();
        unit_ids.sort_by_key(|&UnitId(id)| self.units[id].pos);
        for UnitId(unit_id) in unit_ids {
            let unit = &self.units[unit_id];
            if unit.hp <= 0 {
                continue;
            }

            if let None = self.foe_neighbors(unit.pos, unit.kind).next() {
                if !self.unit_move(UnitId(unit_id)) {
                    return false;
                }
            }

            let foe = self
                .foe_neighbors(self.units[unit_id].pos, self.units[unit_id].kind)
                .map(|UnitId(unit_id)| (&self.units[unit_id], unit_id))
                .min();
            // TODO borrow lifetimes are weird
            if let Some((_, foe)) = foe {
                let foe = &mut self.units[foe];
                foe.hp -= 3;
                if foe.hp <= 0 {
                    self.tiles[foe.pos] = Tile::Open;
                }
            }
        }
        true
    }

    fn remaining_hp(&self) -> u32 {
        self.units.iter().filter(|unit| unit.hp > 0).map(|unit| unit.hp as u32).sum::<u32>()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut units = Vec::new();
        for (i, tile) in self.tiles.iter().enumerate() {
            let c = match tile {
                Tile::Wall => '#',
                Tile::Open => '.',
                &Tile::Unit(UnitId(unit_id)) => {
                    let Unit { kind, hp, .. } = &self.units[unit_id];
                    let c = match kind {
                        UnitKind::Goblin => 'G',
                        UnitKind::Elf => 'E',
                    };
                    units.push((c, hp));
                    c
                }
            };
            write!(f, "{}", c)?;
            if (i + 1) % self.width == 0 {
                if !units.is_empty() {
                    let units_str = units
                        .iter()
                        .map(|(c, hp)| format!("{}({})", c, hp))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "   {}", units_str)?;
                    units.clear();
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut board = Board::from_bytes(io::stdin().bytes())?;
    let mut i = 0;
    while board.next_round() {
        i += 1;
        println!("{}", i);
        if i > 27 { break; }
    }
    println!("{}", board);
    println!("{}", i * board.remaining_hp());
    Ok(())
}
