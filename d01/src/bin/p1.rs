fn try_parse(s: String) -> i32 {
    unimplemented!();
}

fn main() {
    let input = include_str!("../../input");
    let sum: i32 = input.lines().map(|x| x.parse::<i32>().unwrap()).sum();
    println!("{}", sum);
}
