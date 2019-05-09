use failure::{self, bail, format_err};
use regex::Regex;
use std::cmp;
use std::io::{self, Read};
use std::result;
use std::str::FromStr;

type Result<T> = result::Result<T, failure::Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Army {
    Immune,
    Infection,
}

#[derive(Clone)]
struct Group {
    army: Army,
    size: u32,
    hp: u32,
    dmg: u32,
    atk_type: String,
    initiative: u32,
    weaknesses: Vec<String>,
    immunities: Vec<String>,
    boost: u32,
}

impl Group {
    fn effective_power(&self) -> u32 {
        self.size * (self.dmg + self.boost)
    }

    fn damage_to(&self, target: &Group) -> u32 {
        if target.weaknesses.contains(&self.atk_type) {
            self.effective_power() * 2
        } else if target.immunities.contains(&self.atk_type) {
            0
        } else {
            self.effective_power()
        }
    }
}

impl FromStr for Group {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Group> {
        let re = Regex::new(r"^(\d+) units each with (\d+) hit points (\(([\w ;,]+)\) )?with an attack that does (\d+) (\w+) damage at initiative (\d+)$")?;
        let caps = re
            .captures(s)
            .ok_or_else(|| format_err!("Regex did not match"))?;
        let mut weaknesses = Vec::new();
        let mut immunities = Vec::new();
        if let Some(modifiers) = caps.get(4) {
            for modifier in modifiers.as_str().split("; ") {
                let (modified, types) = if modifier.starts_with("immune to ") {
                    (&mut immunities, &modifier[10..])
                } else if modifier.starts_with("weak to ") {
                    (&mut weaknesses, &modifier[8..])
                } else {
                    bail!("modifier did not parse")
                };
                modified.extend(types.split(", ").map(str::to_owned));
            }
        }
        Ok(Group {
            army: Army::Immune, // hacky default
            size: caps[1].parse()?,
            hp: caps[2].parse()?,
            dmg: caps[5].parse()?,
            atk_type: caps[6].to_owned(),
            initiative: caps[7].parse()?,
            weaknesses,
            immunities,
            boost: 0,
        })
    }
}

#[derive(Clone)]
struct Simulation {
    groups: Vec<Group>,
}

impl Simulation {
    fn select_targets(&mut self) -> Vec<Option<usize>> {
        let mut targets = Vec::new();
        // let mut order: Vec<_> = (0..self.groups.len()).collect();
        // order.sort_by_key(|&i| {
        //     cmp::Reverse((self.groups[i].effective_power(), self.groups[i].initiative))
        // });
        // for i in order {
        //     let group = &self.groups[i];
        self.groups.sort_by_key(|g| cmp::Reverse((g.effective_power(), g.initiative)));
        for group in &self.groups {
            println!("{:?} {} units", group.army, group.size);
            let mut candidates = Vec::new();
            for (idx, candidate) in self.groups.iter().enumerate() {
                if group.army == candidate.army || targets.contains(&Some(idx)) {
                    continue;
                }
                let dmg = group.damage_to(&candidate);
                if dmg == 0 {
                    continue;
                }
                println!(" {} dmg to {}", dmg, idx);
                candidates.push((dmg, candidate.effective_power(), candidate.initiative, idx));
            }
            candidates.sort();
            targets.push(candidates.last().map(|&(_, _, _, t)| t));
        }
        targets
    }

    fn attack(&mut self, targets: &Vec<Option<usize>>) {
        let mut order: Vec<_> = (0..self.groups.len()).collect();
        order.sort_by_key(|&i| cmp::Reverse(self.groups[i].initiative));
        for i in order {
            let attacker = &self.groups[i];
            if attacker.size == 0 {
                continue;
            }
            if let Some(target_i) = targets[i] {
                let target = &self.groups[target_i];
                let loss = attacker.damage_to(&target) / target.hp;
                let loss = cmp::min(loss, target.size);
                println!(
                    "{:?} {} attacks {} killing {}",
                    attacker.army, i, target_i, loss
                );
                self.groups[target_i].size -= loss;
            }
        }
        self.groups.retain(|g| g.size > 0);
    }

    fn fight(&mut self) -> bool {
        let targets = self.select_targets();
        self.attack(&targets);
        println!();
        let army = self.groups[0].army;
        return self.groups.iter().any(|g| g.army != army);
    }
}

impl FromStr for Simulation {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Simulation> {
        let mut groups = Vec::new();
        let mut army = Army::Immune;
        for line in s.lines() {
            if line == "Immune System:" {
                army = Army::Immune;
                continue;
            } else if line == "Infection:" {
                army = Army::Infection;
                continue;
            } else if line == "" {
                continue;
            }
            let mut group: Group = line.parse()?;
            group.army = army;
            groups.push(group);
        }
        Ok(Simulation { groups })
    }
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut simulation: Simulation = input.parse()?;
    let orig_simulation = simulation.clone();
    while simulation.fight() {}

    println!("{}", simulation.groups.iter().map(|g| g.size).sum::<u32>());

    let mut boost = 1;
    'outer: loop {
        simulation = orig_simulation.clone();
        for group in &mut simulation.groups {
            if group.army == Army::Immune {
                group.boost = boost;
            }
        }
        let mut num_units = simulation.groups.iter().map(|g| g.size).sum::<u32>();
        while simulation.fight() {
            let new_units = simulation.groups.iter().map(|g| g.size).sum::<u32>();
            if new_units == num_units {
                boost += 1; // count non-terminating fight as a loss
                continue 'outer;
            }
            num_units = new_units;
        }
        match simulation.groups[0].army {
            Army::Immune => break,
            Army::Infection => boost += 1,
        }
    }
    println!("{}", simulation.groups.iter().map(|g| g.size).sum::<u32>());

    Ok(())
}
