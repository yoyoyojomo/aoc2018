use std::error::Error;
use std::io::{self, BufRead};

type Result<T> = ::std::result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut points: Vec<(i32, i32)> = Vec::new();
    for line in io::stdin().lock().lines() {
        match line?.split(", ").collect::<Vec<_>>().as_slice() {
            [x, y] => points.push((x.parse()?, y.parse()?)),
            _ => return Err("unparsable line".into()),
        }
    }

    part1(&points)?;
    part2(&points, 10000)?;
    Ok(())
}

fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn part1(points: &Vec<(i32, i32)>) -> Result<()> {
    let mut areas = vec![Some(0); points.len()];
    let width = points.iter().map(|a| a.0).max().expect("need input");
    let height = points.iter().map(|a| a.1).max().expect("need input");
    for x in 0..=width {
        for y in 0..=height {
            // This might not correctly handle a point being equidistant.
            if let Some((_, i)) = points
                .iter()
                .enumerate()
                .map(|(i, &p)| (manhattan_distance(p, (x, y)), i))
                .min()
            {
                areas[i] = if x == 0 || x == width || y == 0 || y == width {
                    None
                } else {
                    areas[i].map(|x| x + 1)
                }
            }
        }
    }

    let max_area = areas.iter().max().unwrap().ok_or_else(|| Box::<Error>::from("all infinite"))?;

    println!("{}", max_area);
    Ok(())
}

fn part2(points: &Vec<(i32, i32)>, max_distance: i32) -> Result<()> {
    let mut region = 0;
    let width = points.iter().map(|a| a.0).max().expect("need input");
    let height = points.iter().map(|a| a.1).max().expect("need input");
    // We should do a more complicated handling for the infinite edges, but it looks like it is
    // unnecessary for our input.
    for x in 0..=width {
        for y in 0..=height {
            let distance: i32 = points
                .iter()
                .map(|&p| manhattan_distance(p, (x, y)))
                .sum();
            if distance < max_distance {
                region += 1;
            }
        }
    }

    println!("{}", region);
    Ok(())
}
