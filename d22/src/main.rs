use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env;

#[derive(Clone, Copy)]
enum RegionType {
    Rocky,
    Narrow,
    Wet,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Coord(u64, u64);

impl Coord {
    fn manhattan_distance(&self, other: Coord) -> u64 {
        ((self.0 as i64 - other.0 as i64).abs() + (self.1 as i64 - other.1 as i64).abs()) as u64
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Tool {
    Torch,
    Gear,
    Neither,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct State(Reverse<u64>, Tool, Coord);

struct Cave {
    target: Coord,
    depth: u64,
    erosion_cache: RefCell<HashMap<Coord, u64>>,
}

impl Cave {
    fn geologic_index(&self, coord: Coord) -> u64 {
        let Coord(x, y) = coord;
        if coord == Coord(0, 0) || coord == self.target {
            0
        } else if y == 0 {
            x * 16807
        } else if x == 0 {
            y * 48271
        } else {
            self.erosion_level(Coord(x - 1, y)) * self.erosion_level(Coord(x, y - 1))
        }
    }

    fn erosion_level(&self, coord: Coord) -> u64 {
        if let Some(&level) = self.erosion_cache.borrow().get(&coord) {
            return level;
        }
        let level = (self.geologic_index(coord) + self.depth) % 20183;
        self.erosion_cache.borrow_mut().insert(coord, level);
        level
    }

    fn region_type(&self, coord: Coord) -> RegionType {
        match self.erosion_level(coord) % 3 {
            0 => RegionType::Rocky,
            1 => RegionType::Wet,
            2 => RegionType::Narrow,
            _ => unreachable!(),
        }
    }

    fn risk_level(&self, tl: Coord, br: Coord) -> u64 {
        let mut sum = 0;
        for x in tl.0..=br.0 {
            for y in tl.1..=br.1 {
                sum += match self.region_type(Coord(x, y)) {
                    RegionType::Rocky => 0,
                    RegionType::Wet => 1,
                    RegionType::Narrow => 2,
                };
            }
        }
        sum
    }

    fn region_tools(&self, coord: Coord) -> &[Tool; 2] {
        match self.region_type(coord) {
            RegionType::Rocky => &[Tool::Gear, Tool::Torch],
            RegionType::Wet => &[Tool::Gear, Tool::Neither],
            RegionType::Narrow => &[Tool::Torch, Tool::Neither],
        }
    }

    fn explore(&self, state: State, frontier: &mut BinaryHeap<(Reverse<u64>, State)>) {
        let State(Reverse(distance), tool, coord) = state;
        if self.region_tools(coord).contains(&tool) {
            frontier.push((
                Reverse(distance + 1 + coord.manhattan_distance(self.target)),
                State(Reverse(distance + 1), tool, coord),
            ));
        }
    }

    fn astar(&self) -> u64 {
        let mut visited = HashSet::new();
        let mut frontier = BinaryHeap::new();
        frontier.push((
            Reverse(Coord(0, 0).manhattan_distance(self.target)),
            State(Reverse(0), Tool::Torch, Coord(0, 0)),
        ));
        while let Some((_, state)) = frontier.pop() {
            let State(Reverse(distance), tool, Coord(x, y)) = state;
            if !visited.insert((tool, Coord(x, y))) {
                continue;
            }
            if Coord(x, y) == self.target && tool == Tool::Torch {
                return distance;
            }
            self.explore(
                State(Reverse(distance), tool, Coord(x + 1, y)),
                &mut frontier,
            );
            self.explore(
                State(Reverse(distance), tool, Coord(x, y + 1)),
                &mut frontier,
            );
            if x > 0 {
                self.explore(
                    State(Reverse(distance), tool, Coord(x - 1, y)),
                    &mut frontier,
                );
            }
            if y > 0 {
                self.explore(
                    State(Reverse(distance), tool, Coord(x, y - 1)),
                    &mut frontier,
                );
            }
            for &switch_tool in self.region_tools(Coord(x, y)) {
                frontier.push((
                    Reverse(distance + 7 + Coord(x, y).manhattan_distance(self.target)),
                    State(Reverse(distance + 7), switch_tool, Coord(x, y)),
                ));
            }
        }
        unreachable!();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let (depth, target_x, target_y) = match &args.as_slice() {
        &[_, depth, target_x, target_y] => (
            depth.parse().unwrap(),
            target_x.parse().unwrap(),
            target_y.parse().unwrap(),
        ),
        _ => panic!("expected 3 args"),
    };

    let cave = Cave {
        target: Coord(target_x, target_y),
        depth,
        erosion_cache: RefCell::new(HashMap::new()),
    };
    println!("{}", cave.risk_level(Coord(0, 0), cave.target));
    println!("{}", cave.astar());
}
