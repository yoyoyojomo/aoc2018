use failure::{bail, ensure, format_err, Error};
use std::io::{self, Read};
use std::result;
use std::str::FromStr;

type Result<T> = result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum OpCode {
    addr,
    addi,
    mulr,
    muli,
    banr,
    bani,
    borr,
    bori,
    setr,
    seti,
    gtir,
    gtri,
    gtrr,
    eqir,
    eqri,
    eqrr,
}

impl OpCode {
    pub fn variants() -> impl Iterator<Item = OpCode> {
        use OpCode::*;
        static OPCODES: [OpCode; 16] = [
            addr, addi, mulr, muli, banr, bani, borr, bori, setr, seti, gtir, gtri, gtrr, eqir,
            eqri, eqrr,
        ];
        OPCODES.into_iter().cloned()
    }
}

struct Instruction {
    opcode: OpCode,
    in1: u32,
    in2: u32,
    out: u32,
}

struct Machine {
    registers: [u32; 4],
}

impl Machine {
    fn new() -> Machine {
        Machine { registers: [0; 4] }
    }

    fn reg(&self, r: u32) -> u32 {
        self.registers[r as usize]
    }

    fn execute(&mut self, instruction: &Instruction) {
        use OpCode::*;
        let &Instruction {
            opcode,
            in1: a,
            in2: b,
            out,
        } = instruction;
        self.registers[out as usize] = match opcode {
            addr => self.reg(a) + self.reg(b),
            addi => self.reg(a) + b,
            mulr => self.reg(a) * self.reg(b),
            muli => self.reg(a) * b,
            banr => self.reg(a) & self.reg(b),
            bani => self.reg(a) & b,
            borr => self.reg(a) | self.reg(b),
            bori => self.reg(a) | b,
            setr => self.reg(a),
            seti => a,
            gtir => {
                if a > self.reg(b) {
                    1
                } else {
                    0
                }
            }
            gtri => {
                if self.reg(a) > b {
                    1
                } else {
                    0
                }
            }
            gtrr => {
                if self.reg(a) > self.reg(b) {
                    1
                } else {
                    0
                }
            }
            eqir => {
                if a == self.reg(b) {
                    1
                } else {
                    0
                }
            }
            eqri => {
                if self.reg(a) == b {
                    1
                } else {
                    0
                }
            }
            eqrr => {
                if self.reg(a) == self.reg(b) {
                    1
                } else {
                    0
                }
            }
        };
    }
}

struct BlackboxInput {
    before: [u32; 4],
    instruction: [u32; 4],
    after: [u32; 4],
}

impl FromStr for BlackboxInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<BlackboxInput> {
        let lines: Vec<_> = s.split("\n").collect();
        ensure!(lines.len() == 3, "paragraph");

        ensure!(
            lines[0].starts_with("Before: [") && lines[0].ends_with("]"),
            "before"
        );
        let before: Vec<u32> = lines[0][9..lines[0].len() - 1]
            .split(", ")
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;
        let before = match before.as_slice() {
            &[a, b, c, d] => [a, b, c, d],
            _ => bail!("before"),
        };

        let instruction: Vec<u32> = lines[1]
            .split(" ")
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;
        let instruction = match instruction.as_slice() {
            &[a, b, c, d] => [a, b, c, d],
            _ => bail!("instruction"),
        };

        ensure!(
            lines[2].starts_with("After:  [") && lines[2].ends_with("]"),
            "after"
        );
        let after: Vec<u32> = lines[2][9..lines[2].len() - 1]
            .split(", ")
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;
        let after = match after.as_slice() {
            &[a, b, c, d] => [a, b, c, d],
            _ => bail!("after"),
        };

        Ok(BlackboxInput {
            before,
            instruction,
            after,
        })
    }
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut input = input.split("\n\n\n\n");
    let part1 = input.next().ok_or_else(|| format_err!("bad input"))?;
    let part2 = input.next().ok_or_else(|| format_err!("bad input"))?;

    let part1: Vec<BlackboxInput> = part1
        .split("\n\n")
        .map(|s| s.parse())
        .collect::<result::Result<_, _>>()?;

    let mut opcode_candidates: [Vec<OpCode>; 16] = Default::default();
    for candidates in &mut opcode_candidates {
        candidates.extend(OpCode::variants());
    }
    let mut behaves_like_3 = 0;
    let mut machine = Machine::new();
    for BlackboxInput {
        before,
        instruction,
        after,
    } in part1
    {
        let mut matching_opcodes = Vec::new();
        for opcode in OpCode::variants() {
            machine.registers = before;
            let instruction = Instruction {
                opcode,
                in1: instruction[1],
                in2: instruction[2],
                out: instruction[3],
            };
            machine.execute(&instruction);
            if machine.registers == after {
                matching_opcodes.push(opcode);
            }
        }
        opcode_candidates[instruction[0] as usize].retain(|c| matching_opcodes.contains(c));
        if matching_opcodes.len() >= 3 {
            behaves_like_3 += 1;
        }
    }

    println!("{}", behaves_like_3);

    while opcode_candidates.iter().any(|c| c.len() > 1) {
        let mut resolved = Vec::new();
        for candidates in &opcode_candidates {
            if candidates.len() == 1 {
                resolved.extend_from_slice(&candidates);
            }
        }
        for i in 0..opcode_candidates.len() {
            if opcode_candidates[i].len() == 1 {
                continue;
            }
            opcode_candidates[i].retain(|c| !resolved.contains(&c));
        }
    }

    let mut machine = Machine::new();
    for line in part2.lines() {
        let instruction: Vec<u32> = line
            .split(" ")
            .map(|s| s.parse())
            .collect::<result::Result<_, _>>()?;
        let instruction = match instruction.as_slice() {
            &[opcode, in1, in2, out] => Instruction {
                opcode: opcode_candidates[opcode as usize][0],
                in1,
                in2,
                out,
            },
            _ => bail!("instruction"),
        };
        machine.execute(&instruction);
    }

    println!("{}", machine.reg(0));

    Ok(())
}
