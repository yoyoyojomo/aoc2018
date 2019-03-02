use std::env;
use std::error::Error;
use std::fmt::{self, Debug};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

const SEGMENT_SIZE: usize = 1024;

struct MarbleSegment {
    marbles: [u32; SEGMENT_SIZE],
    len: usize,
}

impl MarbleSegment {
    fn new() -> Self {
        Self {
            marbles: [0; SEGMENT_SIZE],
            len: 1,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, n: usize) -> u32 {
        self.marbles[n]
    }

    fn needs_split(&self) -> bool {
        self.len == SEGMENT_SIZE
    }

    fn insert(&mut self, n: usize, marble: u32) {
        assert!(self.len < SEGMENT_SIZE);
        assert!(n <= self.len);
        for i in (n..self.len).rev() {
            self.marbles[i + 1] = self.marbles[i];
        }
        self.marbles[n] = marble;
        self.len += 1;
    }

    fn remove(&mut self, n: usize) {
        assert!(self.len > 0);
        for i in n + 1..self.len {
            self.marbles[i - 1] = self.marbles[i];
        }
        self.len -= 1;
    }

    fn split_off(&mut self) -> MarbleSegment {
        let split_at = self.len / 2;
        let mut marbles = [0; SEGMENT_SIZE];
        marbles[..self.len - split_at].copy_from_slice(&self.marbles[split_at..self.len]);
        let len = self.len - split_at;
        self.len = split_at;
        Self { marbles, len }
    }
}

impl Debug for MarbleSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.marbles[..self.len])
    }
}

struct MarbleRing {
    // This should be a circular linked list but that's surprisingly hard with Rust ownership.
    segments: Vec<MarbleSegment>,
    current_segment: usize,
    segment_index: usize,
    len: usize,
}

impl MarbleRing {
    fn new() -> Self {
        Self {
            segments: vec![MarbleSegment::new()],
            current_segment: 0,
            segment_index: 0,
            len: 0,
        }
    }

    fn ccw_by(&mut self, mut n: usize) {
        while n > self.segment_index {
            n -= self.segment_index + 1;
            loop {
                self.current_segment = if self.current_segment == 0 {
                    self.segments.len() - 1
                } else {
                    self.current_segment - 1
                };
                if self.segments[self.current_segment].len() > 0 {
                    break;
                }
            }
            self.segment_index = self.segments[self.current_segment].len() - 1;
        }
        self.segment_index -= n;
    }

    fn cw_by(&mut self, n: usize) {
        self.segment_index += n;
        while self.segment_index > self.segments[self.current_segment].len() {
            self.segment_index -= self.segments[self.current_segment].len();
            self.current_segment += 1;
            self.current_segment %= self.segments.len();
        }
    }

    fn get(&self) -> u32 {
        self.segments[self.current_segment].get(self.segment_index)
    }

    fn insert(&mut self, marble: u32) {
        if self.segments[self.current_segment].needs_split() {
            let split_segment = self.segments[self.current_segment].split_off();
            self.segments
                .insert(self.current_segment + 1, split_segment);
            if self.segment_index >= self.segments[self.current_segment].len() {
                self.segment_index -= self.segments[self.current_segment].len();
                self.current_segment += 1;
                self.current_segment %= self.segments.len();
            }
        }
        self.segments[self.current_segment].insert(self.segment_index, marble);
        self.len += 1;
    }

    fn remove(&mut self) {
        self.segments[self.current_segment].remove(self.segment_index);
        while self.segments[self.current_segment].len() == 0 {
            // hack to avoid removing segments
            self.current_segment = (self.current_segment + 1) % self.segments.len();
        }
        self.len -= 1;
    }
}

impl Debug for MarbleRing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (segment_i, segment) in self.segments.iter().enumerate() {
            for i in 0..segment.len() {
                if segment_i == self.current_segment && i == self.segment_index {
                    write!(f, "*")?;
                }
                write!(f, "{} ", segment.get(i))?;
            }
        }
        Ok(())
    }
}

struct MarbleGame {
    last_marble: u32,
    ring: MarbleRing,
}

impl MarbleGame {
    fn new() -> Self {
        Self {
            last_marble: 0,
            ring: MarbleRing::new(),
        }
    }

    fn place_next(&mut self) -> u32 {
        self.last_marble += 1;
        if self.last_marble % 23 == 0 {
            self.ring.ccw_by(7);
            let score = self.last_marble + self.ring.get();
            self.ring.remove();
            score
        } else {
            self.ring.cw_by(2);
            self.ring.insert(self.last_marble);;
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

    let mut game = MarbleGame::new();
    let mut scores = vec![0; num_players];
    let mut turn = 0;
    for _ in 0..last_marble {
        scores[turn] += game.place_next();
        turn = (turn + 1) % num_players;
    }
    println!("{}", scores.iter().max().ok_or("need players")?);
    Ok(())
}
