use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::result;
use std::usize;

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Clone, PartialEq)]
enum Tile {
    Wall,
    Open,
    Unit,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum UnitKind {
    Goblin,
    Elf,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Unit {
    hp: u32,
    attack: u32,
    kind: UnitKind,
    id: usize,
}

#[derive(Clone)]
struct Board {
    tiles: Vec<Tile>,
    units: BTreeMap<usize, Unit>,
    width: usize,
    elf_attack: u32,
    elf_casualty: bool,
}

impl Board {
    fn from_bytes(bytes: io::Bytes<io::Stdin>) -> Result<Board> {
        let mut tiles = Vec::new();
        let mut units = BTreeMap::new();
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
                    units.insert(
                        tiles.len(),
                        Unit {
                            hp: 200,
                            attack: 3,
                            kind: match byte {
                                b'E' => UnitKind::Elf,
                                b'G' | _ => UnitKind::Goblin,
                            },
                            id: units.len(),
                        },
                    );
                    Tile::Unit
                }
                _ => return Err("invalid byte".into()),
            };
            tiles.push(tile);
        }
        Ok(Board {
            tiles,
            units,
            width,
            elf_attack: 3,
            elf_casualty: false,
        })
    }

    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> {
        // Assumes board bordered by walls.
        vec![pos - self.width, pos - 1, pos + 1, pos + self.width].into_iter()
    }

    fn open_neighbors(&self, pos: usize) -> impl Iterator<Item = usize> {
        self.neighbors(pos)
            .filter(|&pos| self.tiles[pos] == Tile::Open)
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn enemy_neighbors(&self, pos: usize, kind: UnitKind) -> impl Iterator<Item = usize> {
        self.neighbors(pos)
            .filter(|pos| match self.units.get(pos) {
                Some(unit) => unit.kind != kind,
                None => false,
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn bfs_step(&self, src: usize, dst: Vec<usize>) -> Option<usize> {
        let mut distances = vec![usize::MAX; self.tiles.len()];
        let mut max_distance = usize::MAX;
        let mut horizon = VecDeque::new();
        horizon.push_back((0, src));
        while let Some((distance, pos)) = horizon.pop_front() {
            if distance > max_distance {
                break;
            }
            if distance >= distances[pos] {
                continue;
            } else {
                distances[pos] = distance;
            }
            if dst.contains(&pos) {
                max_distance = distance;
            }
            for neighbor in self.open_neighbors(pos) {
                horizon.push_back((distance + 1, neighbor));
            }
        }

        let position = dst
            .into_iter()
            .filter(|&d| distances[d] == max_distance)
            .min()
            .unwrap();
        let mut positions = vec![position];
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

    fn attack_for(&self, unit: &Unit) -> u32 {
        match unit.kind {
            UnitKind::Goblin => 3,
            UnitKind::Elf => self.elf_attack,
        }
    }

    fn next_round(&mut self) -> bool {
        let units: Vec<_> = self.units.keys().cloned().map(|p| (p, self.units[&p].id)).collect();
        for (mut pos, id) in units {
            let unit = self.units.get(&pos);
            if unit.map(|u| u.id != id).unwrap_or(true) {
                continue;
            }
            let unit = unit.unwrap();

            let targets: Vec<_> = self
                .units
                .iter()
                .filter(|(_, target)| target.kind != unit.kind)
                .collect();
            if targets.is_empty() {
                return false;
            }

            let mut targets: Vec<_> = targets
                .iter()
                .flat_map(|&(&pos, _)| self.neighbors(pos))
                .filter(|&pos| self.tiles[pos] == Tile::Open)
                .collect();
            targets.sort();
            targets.dedup();

            if let None = self.enemy_neighbors(pos, unit.kind).next() {
                if targets.is_empty() {
                    continue;
                }
                if let Some(next_pos) = self.bfs_step(pos, targets) {
                    self.tiles.swap(pos, next_pos);
                    let unit = self.units.remove(&pos).unwrap();
                    self.units.insert(next_pos, unit);
                    pos = next_pos;
                }
            }

            let unit = &self.units[&pos];
            let enemy = self
                .enemy_neighbors(pos, unit.kind)
                .map(|pos| (&self.units[&pos], pos))
                .min();
            if let Some((enemy, enemy_pos)) = enemy {
                let attack = self.attack_for(unit);
                if enemy.hp <= attack {
                    if enemy.kind == UnitKind::Elf {
                        self.elf_casualty = true;
                    }
                    self.tiles[enemy_pos] = Tile::Open;
                    self.units.remove(&enemy_pos);
                } else {
                    self.units.get_mut(&enemy_pos).unwrap().hp -= attack;
                }
            }
        }
        true
    }

    fn remaining_hp(&self) -> u32 {
        self.units.values().map(|unit| unit.hp).sum()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut units = Vec::new();
        for (i, tile) in self.tiles.iter().enumerate() {
            let c = match tile {
                Tile::Wall => '#',
                Tile::Open => '.',
                Tile::Unit => {
                    let Unit { kind, hp, .. } = &self.units[&i];
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
    let orig_board = Board::from_bytes(io::stdin().bytes())?;

    let mut board = orig_board.clone();
    let mut i = 0;
    while board.next_round() {
        i += 1;
    }
    println!("{}", i * board.remaining_hp());

    'outer: for attack in 4.. {
        let mut board = orig_board.clone();
        board.elf_attack = attack;
        let mut i = 0;
        while board.next_round() {
            if board.elf_casualty {
                continue 'outer;
            }
            i += 1;
        }
        println!("{}", i * board.remaining_hp());
        break;
    }

    Ok(())
}
