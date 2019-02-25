use std::collections::HashSet;
use std::io::{self, Read};

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn units_react(x: u8, y: u8) -> bool {
    x.to_ascii_uppercase() == y.to_ascii_uppercase()
        && x.is_ascii_uppercase() != y.is_ascii_uppercase()
}

fn react_polymer<T>(polymer: T) -> Result<Vec<u8>>
where
    T: Iterator<Item = ::std::result::Result<u8, ::std::io::Error>>,
{
    let mut reacted = Vec::new();
    for unit in polymer {
        reacted.push(unit?);
        while reacted.len() >= 2
            && units_react(reacted[reacted.len() - 1], reacted[reacted.len() - 2])
        {
            reacted.truncate(reacted.len() - 2);
        }
    }
    Ok(reacted)
}

#[test]
fn test_react_polymer() -> Result<()> {
    assert_eq!(react_polymer("foo".as_bytes().bytes())?, "foo".as_bytes());
    assert_eq!(react_polymer("foO".as_bytes().bytes())?, "f".as_bytes());
    assert_eq!(react_polymer("foOFoo".as_bytes().bytes())?, "oo".as_bytes());
    assert_eq!(
        react_polymer("dabAcCaCBAcCcaDA".as_bytes().bytes())?,
        "dabCBAcaDA".as_bytes()
    );
    Ok(())
}

fn remove_unit(polymer: &Vec<u8>, unit: u8) -> Vec<u8> {
    polymer
        .into_iter()
        .map(|x| *x)
        .filter(|u| u.to_ascii_uppercase() != unit)
        .collect()
}

fn main() -> Result<()> {
    let mut polymer = react_polymer(io::stdin().lock().bytes())?;
    while polymer.last() == Some(&b'\n') {
        polymer.pop();
    }
    println!("{}", polymer.len());

    let mut units = HashSet::new();
    for unit in &polymer {
        units.insert(unit.to_ascii_uppercase());
    }

    let minimized_length = units
        .into_iter()
        .map(|unit| {
            react_polymer(remove_unit(&polymer, unit).bytes())
                .unwrap()
                .len()
        })
        .min()
        .unwrap();
    println!("{}", minimized_length);

    Ok(())
}
