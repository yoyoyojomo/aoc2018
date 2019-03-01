use std::error::Error;
use std::io::{self, Read};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

struct Node {
    children: Box<[Node]>,
    metadata: Box<[usize]>,
}

fn parse_tree<T: Iterator<Item = usize>>(it: &mut T) -> Result<Node> {
    let num_children = it
        .next()
        .ok_or_else(|| Box::<Error>::from("malformed header"))?;
    let num_metadata = it
        .next()
        .ok_or_else(|| Box::<Error>::from("malformed header"))?;
    let mut children = Vec::new();
    let mut metadata = Vec::new();
    for _ in 0..num_children {
        children.push(parse_tree(it)?);
    }
    for _ in 0..num_metadata {
        metadata.push(
            it.next()
                .ok_or_else(|| Box::<Error>::from("missing metadata"))?,
        );
    }
    let children = children.into_boxed_slice();
    let metadata = metadata.into_boxed_slice();
    Ok(Node { children, metadata })
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input
        .trim()
        .split(' ')
        .map(str::parse)
        .collect::<result::Result<Vec<usize>, _>>()?;
    let root = parse_tree(&mut input.into_iter())?;

    part1(&root)?;
    part2(&root)?;
    Ok(())
}

fn sum_metadata(node: &Node) -> usize {
    node.children.iter().map(sum_metadata).sum::<usize>() + node.metadata.iter().sum::<usize>()
}

fn part1(root: &Node) -> Result<()> {
    println!("{}", sum_metadata(root));
    Ok(())
}

fn get_value(node: &Node) -> Result<usize> {
    if node.children.is_empty() {
        Ok(node.metadata.iter().sum())
    } else {
        let mut sum = 0;
        for i in node.metadata.iter().cloned() {
            if i != 0 && i <= node.children.len() {
                sum += get_value(&node.children[i - 1])?;
            }
        }
        Ok(sum)
    }
}

fn part2(root: &Node) -> Result<()> {
    println!("{}", get_value(root)?);
    Ok(())
}
