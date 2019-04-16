use failure::{bail, ensure, Error};
use std::collections::HashSet;
use std::fmt;
use std::io::{self, BufRead};
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

impl FromStr for OpCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "addr" => Ok(OpCode::addr),
            "addi" => Ok(OpCode::addi),
            "mulr" => Ok(OpCode::mulr),
            "muli" => Ok(OpCode::muli),
            "banr" => Ok(OpCode::banr),
            "bani" => Ok(OpCode::bani),
            "borr" => Ok(OpCode::borr),
            "bori" => Ok(OpCode::bori),
            "setr" => Ok(OpCode::setr),
            "seti" => Ok(OpCode::seti),
            "gtir" => Ok(OpCode::gtir),
            "gtri" => Ok(OpCode::gtri),
            "gtrr" => Ok(OpCode::gtrr),
            "eqir" => Ok(OpCode::eqir),
            "eqri" => Ok(OpCode::eqri),
            "eqrr" => Ok(OpCode::eqrr),
            _ => bail!("unknown opcode"),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            OpCode::addr => "addr",
            OpCode::addi => "addi",
            OpCode::mulr => "mulr",
            OpCode::muli => "muli",
            OpCode::banr => "banr",
            OpCode::bani => "bani",
            OpCode::borr => "borr",
            OpCode::bori => "bori",
            OpCode::setr => "setr",
            OpCode::seti => "seti",
            OpCode::gtir => "gtir",
            OpCode::gtri => "gtri",
            OpCode::gtrr => "gtrr",
            OpCode::eqir => "eqir",
            OpCode::eqri => "eqri",
            OpCode::eqrr => "eqrr",
        };
        write!(f, "{}", s)
    }
}

struct Instruction {
    opcode: OpCode,
    in1: u64,
    in2: u64,
    out: u64,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.opcode, self.in1, self.in2, self.out)
    }
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut words = s.split(' ');
        let opcode = words.next().expect("opcode").parse()?;
        let in1 = words.next().expect("in1").parse()?;
        let in2 = words.next().expect("in2").parse()?;
        let out = words.next().expect("out").parse()?;
        ensure!(words.next().is_none(), "spurious input");
        Ok(Instruction {
            opcode,
            in1,
            in2,
            out,
        })
    }
}

struct Machine {
    registers: [u64; 6],
    bindip: usize,
    instructions: Vec<Instruction>,
}

fn bool_to_u64(b: bool) -> u64 {
    if b {
        1
    } else {
        0
    }
}

impl Machine {
    fn new(bindip: usize, instructions: Vec<Instruction>) -> Machine {
        Machine {
            registers: [0; 6],
            bindip,
            instructions,
        }
    }

    // fn reset(&mut self) {
    //     self.registers = [0; 6];
    // }

    fn reg(&self, r: u64) -> u64 {
        self.registers[r as usize]
    }

    fn ip(&self) -> usize {
        self.registers[self.bindip] as usize
    }

    fn execute(&mut self) {
        use OpCode::*;
        let Instruction {
            opcode,
            in1: a,
            in2: b,
            out,
        } = self.instructions[self.ip()];
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
            gtir => bool_to_u64(a > self.reg(b)),
            gtri => bool_to_u64(self.reg(a) > b),
            gtrr => bool_to_u64(self.reg(a) > self.reg(b)),
            eqir => bool_to_u64(a == self.reg(b)),
            eqri => bool_to_u64(self.reg(a) == b),
            eqrr => bool_to_u64(self.reg(a) == self.reg(b)),
        };
    }

    fn step(&mut self) -> Option<()> {
        self.execute();
        self.registers[self.bindip] += 1;
        if self.ip() < self.instructions.len() {
            Some(())
        } else {
            None
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let bindip = lines.next().expect("empty input")?;
    ensure!(bindip.starts_with("#ip "), "#ip");
    let bindip = bindip[4..].parse()?;
    let instructions: Vec<Instruction> = lines
        .map(|l| l.unwrap().parse())
        .collect::<Result<Vec<Instruction>>>()?;

    let mut m = Machine::new(bindip, instructions);
    m.registers[0] = 11285115;
    println!("#ip {}", bindip);
    let mut exit = false;
    while !exit {
        // let prefix = format!("ip={} {:?} {}", m.ip(), m.registers, m.instructions[m.ip()]);
        exit = m.step().is_none();
        // println!("{} {:?}", prefix, m.registers);
    }

    let mut candidates = HashSet::new();
    let mut r = [0u64; 6];
    'i6: loop {
        // 06 bori 1 65536 3
        r[3] = r[1] | 65536;
        // 07 seti 10905776 4 1
        r[1] = 10905776;
        'i8: loop {
            // 08 bani 3 255 4
            r[4] = r[3] & 255;
            // 09 addr 1 4 1
            r[1] += r[4];
            // 10 bani 1 16777215 1
            r[1] &= 16777215;
            // 11 muli 1 65899 1
            r[1] *= 65899;
            // 12 bani 1 16777215 1
            r[1] &= 16777215;
            // 13 gtir 256 3 4
            // 14 addr 4 2 2
            // 16 seti 27 1 2
            if 256 > r[3] {
                if !candidates.insert(r[1]) {
                    break 'i6;
                }
                println!("{}", r[1]);
                // 28 eqrr 1 0 4
                // 29 addr 4 2 2
                if r[1] == r[0] {
                    break 'i6;
                }
                // 30 seti 5 1 2
                continue 'i6;
            }

            // // 15 addi 2 1 2
            // // 17 seti 0 6 4
            // r[4] = 0;
            // loop {
            //     // 18 addi 4 1 5
            //     r[5] = r[4] + 1;
            //     // 19 muli 5 256 5
            //     r[5] *= 256;
            //     // 20 gtrr 5 3 5
            //     // 21 addr 5 2 2
            //     // 23 seti 25 1 2
            //     if r[5] > r[3] {
            //         // 26 setr 4 7 3
            //         r[3] = r[4];
            //         // 27 seti 7 4 2
            //         continue 'i8;
            //     }
            //     // 22 addi 2 1 2
            //     // 24 addi 4 1 4
            //     r[4] += 1;
            //     // 25 seti 17 9 2
            // }
            r[3] = r[3] / 256;
        }
    }

    Ok(())
}
