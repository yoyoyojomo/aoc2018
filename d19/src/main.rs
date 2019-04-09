use failure::{bail, ensure, Error};
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

    fn reset(&mut self) {
        self.registers = [0; 6];
    }

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

    // The input assembly slowly adds all divisors.
    fn shortcut(&mut self) {
        match self.registers {
            [_, _, _, 3, _, _] if self.reg(5) * self.reg(4) < self.reg(1) => {
                self.registers[4] = self.reg(1) / self.reg(5);
                println!("set reg 4 to {}", self.reg(4));
            }
            [_, _, _, 9, _, _] if self.reg(4) <= self.reg(1) && self.reg(5) > 1 => {
                self.registers[4] = self.reg(1) + 1;
                println!("set reg 4 to {}", self.reg(4));
            }
            [_, _, _, 13, _, _]
                if self.reg(5) > 2
                    && self.reg(5) < self.reg(1)
                    && (self.reg(1) - self.reg(5)) % 2 == 0
                    && self.reg(4) == self.reg(1) + 1 =>
            {
                let sum: u64 = (self.reg(5)..self.reg(1))
                    .filter(|x| self.reg(1) % x == 0)
                    .sum();
                self.registers[0] = self.reg(0) + sum;
                self.registers[5] = self.reg(1);
                println!("set reg 0 to {}, 5 to {}", self.reg(0), self.reg(5));
            }
            _ => {}
        }
    }

    fn step(&mut self) -> Option<()> {
        self.shortcut();
        self.execute();
        self.registers[self.bindip] += 1;
        if self.ip() < self.instructions.len() {
            Some(())
        } else {
            None
        }
    }

    fn run(&mut self) {
        while let Some(()) = self.step() {}
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
    m.run();
    println!("{}", m.registers[0]);

    m.reset();
    m.registers[0] = 1;
    println!("#ip {}", bindip);
    let mut exit = false;
    while !exit {
        let prefix = format!("ip={} {:?} {}", m.ip(), m.registers, m.instructions[m.ip()]);
        exit = m.step().is_none();
        println!("{} {:?}", prefix, m.registers);
    }
    println!("{}", m.registers[0]);

    Ok(())
}
