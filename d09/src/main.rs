use std::result;
use std::error::Error;
use std::env;

type Result<T> = result::Result<T, Box<Error>>;

struct MarbleRing {
    marbles: Vec<u32>,
    last_marble: u32,
    current_idx: usize,
}

impl MarbleRing {
    fn new() -> Self {
        Self {
            marbles: vec![0],
            last_marble: 0,
            current_idx: 0,
        }
    }

    fn clockwise_by(&self, mut n: i32) -> usize {
        while n < 0 {
            n += self.marbles.len() as i32;
        }
        (self.current_idx + n as usize) % self.marbles.len()
    }

    fn place_next(&mut self) -> u32 {
        self.last_marble += 1;
        if self.last_marble % 23 == 0 {
            let remove_from = self.clockwise_by(-7);
            let score = self.last_marble + self.marbles[remove_from];
            self.marbles.remove(remove_from);
            self.current_idx = remove_from % self.marbles.len();
            score
        } else {
            self.current_idx = self.clockwise_by(2);
            self.marbles.insert(self.current_idx, self.last_marble);
            0
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args();
    args.next();
    let num_players: usize = args.next().ok_or("missing num players")?.parse()?;
    let last_marble: u32 = args.next().ok_or("missing last marble")?.parse()?;
    if args.next() != None {
        return Err("expected 2 arguments".into());
    }

    let mut ring = MarbleRing::new();
    let mut scores = vec![0; num_players];
    let mut turn = 0;
    for _ in 0..last_marble {
        scores[turn] += ring.place_next();
        turn = (turn + 1) % num_players;
    }
    println!("{}", scores.iter().max().ok_or("need players")?);
    Ok(())
}
