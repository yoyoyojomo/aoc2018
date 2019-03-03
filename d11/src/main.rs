use std::env;
use std::error::Error;
use std::i32;
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;

#[derive(Clone, Copy)]
struct PowerCell(i32);

impl PowerCell {
    fn generate(x: usize, y: usize, serial_num: i32) -> Self {
        let (x, y) = (x as i32, y as i32);
        let rack_id = x + 10;
        let mut result = rack_id * y;
        result += serial_num;
        result *= rack_id;
        let result = (result / 100) % 10;
        Self(result - 5)
    }
}

struct Grid {
    sums: [[i32; WIDTH + 1]; HEIGHT + 1],
}

impl Grid {
    fn from_serial_num(serial_num: i32) -> Self {
        let mut power_cells = [[PowerCell(0); WIDTH + 1]; HEIGHT + 1];
        for y in 1..=HEIGHT {
            for x in 1..=WIDTH {
                power_cells[y][x] = PowerCell::generate(x, y, serial_num);
            }
        }

        let mut sums = [[0; WIDTH + 1]; HEIGHT + 1];
        for y in 1..=HEIGHT {
            for x in 1..=WIDTH {
                sums[y][x] =
                    sums[y][x - 1] + sums[y - 1][x] - sums[y - 1][x - 1] + power_cells[y][x].0;
            }
        }

        Self { sums }
    }

    fn max(&self, size: usize) -> (i32, usize, usize) {
        let mut result = (i32::MIN, 1, 1);
        for r in 0..=HEIGHT - size {
            for c in 0..=WIDTH - size {
                let power = self.sums[r + size][c + size] + self.sums[r][c]
                    - self.sums[r + size][c]
                    - self.sums[r][c + size];
                if power > result.0 {
                    result = (power, c + 1, r + 1);
                }
            }
        }
        result
    }
}

fn main() -> Result<()> {
    let mut args = env::args();
    args.next();
    let serial_num: i32 = args.next().ok_or("missing serial_num")?.parse()?;
    if args.next() != None {
        return Err("expected 1 argument".into());
    }

    let grid = Grid::from_serial_num(serial_num);
    let (_, x, y) = grid.max(3);
    println!("{},{}", x, y);

    let ((_, x, y), size) = (1..WIDTH).map(|size| (grid.max(size), size)).max().unwrap();
    println!("{},{},{}", x, y, size);
    Ok(())
}
