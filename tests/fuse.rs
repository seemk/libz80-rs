#![feature(globs)]
#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate z80;
use std::io::{File, BufferedReader};
use std::str::*;
use std::{u8, u16, u32};
use std::vec::*;
use std::iter::range_step;

use z80::UserData;
use std::mem::transmute;

struct FuseEmulator {
    pub cpu: z80::Context,
    pub mem: Vec<u8> 
}

impl FuseEmulator {
  
    pub fn new() -> FuseEmulator {
        FuseEmulator {
            cpu: z80::Context::new(),
            mem: Vec::from_elem(0x10000u, 0u8)
        }
    }

    fn read(&self, address: u16) -> u8 {
        *self.mem.get(address as uint)    
    }

    fn write(&mut self, address: u16, value: u8) {
        *self.mem.get_mut(address as uint) = value;
    }
}

fn split_by_whitespace<'a>(line: &'a str) -> Vec<&'a str> {
    let re = regex!(r"[ \t]+");
    re.split(line).collect()
}

fn parse_word(raw: &Vec<&str>, idx: uint) -> u16 {
    u16::parse_bytes(raw.get(idx).as_bytes(), 16).unwrap()
}

fn parse_byte(raw: &Vec<&str>, idx: uint, base: uint) -> u8 {
    u8::parse_bytes(raw.get(idx).as_bytes(), base).unwrap()
}

fn parse_general_regs(ctx: &mut z80::Context, line: &str) {
    
    let regs = split_by_whitespace(line);

    ctx.write_AF1(parse_word(&regs, 0));
    ctx.write_BC1(parse_word(&regs, 1));
    ctx.write_DE1(parse_word(&regs, 2));
    ctx.write_HL1(parse_word(&regs, 3));
    ctx.write_AF2(parse_word(&regs, 4));
    ctx.write_BC2(parse_word(&regs, 5));
    ctx.write_DE2(parse_word(&regs, 6));
    ctx.write_HL2(parse_word(&regs, 7));

    ctx.write_IX(parse_word(&regs, 8));
    ctx.write_IY(parse_word(&regs, 9));
    ctx.write_SP(parse_word(&regs, 10));
    ctx.write_PC(parse_word(&regs, 11));
}

fn parse_extra_regs(ctx: &mut z80::Context, line: &str) -> u32 {

    let regs = split_by_whitespace(line);

    ctx.write_I(parse_byte(&regs, 0, 16));
    ctx.write_R(parse_byte(&regs, 1, 16));
    ctx.write_IFF1(parse_byte(&regs, 2, 10));
    ctx.write_IFF2(parse_byte(&regs, 3, 10));
    ctx.write_IM(parse_byte(&regs, 4, 10));
    ctx.write_halted(parse_byte(&regs, 5, 10));
    u32::parse_bytes(regs.get(6).as_bytes(), 10).unwrap()
}

fn parse_memory(mem: &mut [u8], line: &str) {
    let mem_bytes = split_by_whitespace(line);
    let mut address = u32::parse_bytes(mem_bytes.get(0).as_bytes(), 16).unwrap();
    for mem_str in mem_bytes.slice(1, mem_bytes.len()).iter() {
        if mem_str.as_slice() != "-1" {
            let mem_val = u8::parse_bytes(mem_str.as_bytes(), 16).unwrap();
            mem[address as uint] = mem_val;
            address += 1;
        }
    }
}

fn dump_context(ctx: &mut z80::Context) {
    print!("{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} \
    {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}\n\
    {:02x} {:02x} {:u} {:u} {:u} {:u} {:u}\n",
    ctx.read_AF1(), ctx.read_BC1(), ctx.read_DE1(), ctx.read_HL1(),
    ctx.read_AF2(), ctx.read_BC2(), ctx.read_DE2(), ctx.read_HL2(),
    ctx.read_IX(), ctx.read_IY(), ctx.read_SP(), ctx.read_PC(),
    ctx.read_I(), ctx.read_R(), ctx.read_IFF1(), ctx.read_IFF2(),
    ctx.read_IM(), ctx.read_halted(), ctx.read_tstates());
}

fn dump_memory(new_mem: &[u8], initial_mem: &[u8]) {

  let mut iter = range(0u, new_mem.len());
  let mut mem_line = false;
  for i in iter {
    let addr = i as u16;
    if new_mem[i] != initial_mem[i] { 
        if !mem_line {
            print!("{:04x} ", addr);
            mem_line = true;
        }
        print!("{:02x} ", new_mem[i]);
    } else {
        if mem_line {
            mem_line = false; 
            println!("-1");
        }
    }
  }

  if mem_line {
    println!("-1");
  }
}

extern fn io_in(_: i32, addr: u16, _: *mut UserData ) -> u8 {
    let data = (addr >> 8) as u8;
    println!("PR {:04x} {:02x}", addr, data);
    data
}

extern fn io_out(_: i32, addr: u16, data: u8, _: *mut UserData) {
    println!("PW {:04x} {:02x}", addr, data);
}


extern fn mem_read(_: i32, addr: u16, user_data: *mut UserData) -> u8 {
    unsafe {
        let emulator: &mut FuseEmulator = transmute(user_data);
        emulator.read(addr)
    }
}

extern fn mem_write(_: i32, addr: u16, data: u8, user_data: *mut UserData) {
    unsafe {
        let emulator: &mut FuseEmulator = transmute(user_data);
        emulator.write(addr, data)
    }
}

fn main() {

    let reset_emulator = || {
        let mut emu = box FuseEmulator::new();
        emu.cpu.set_io_read_callback(Some(io_in));
        emu.cpu.set_io_write_callback(Some(io_out));
        emu.cpu.set_mem_read_callback(Some(mem_read));
        emu.cpu.set_mem_write_callback(Some(mem_write));

        for i in range_step(0u, emu.mem.len(), 4) {
            let addr = i as u16;
            emu.write(addr, 0xdeu8);
            emu.write(addr+1, 0xadu8);
            emu.write(addr+2, 0xbeu8);
            emu.write(addr+3, 0xefu8);
        }

        unsafe {
            let usr_data: *mut UserData = transmute(&mut *emu);
            emu.cpu.set_memory_accessor(usr_data);
        }
        emu
    };

    enum ParseState {
        Description, // Test's identifier
        GeneralRegs, // AF BC DE HL AF' BC' DE' HL' IX IY SP PC
        ExtraRegs,   // I R IFF1 IFF2 IM <halted> <tstates>
        Memory       // <start addr> bytes -1
    }
   
    let mut emu = reset_emulator();
    let tests_path = Path::new("fuse_files/tests.in");

    let mut file = BufferedReader::new(File::open(&tests_path));

    let lines: Vec<String> = file.lines().map(|l|
        String::from_str(l.unwrap().as_slice().trim())
    ).filter(|s| !s.is_empty()).collect();

    let mut iter = lines.iter();
    let mut parse_state = Description;


    let mut description: &String = &String::new();
    let mut end_tstates = 0u32;
    let mut initial_memory: Vec<u8>;

    loop {

        let line = match iter.next() {
            Some(s) => s,
            None => break
        };

        if line.as_slice() == "-1" {

            initial_memory = emu.mem.clone();
            
            println!("{}", &description);
            while emu.cpu.read_tstates() < end_tstates {
                emu.cpu.execute();
            }
            
            dump_context(&mut emu.cpu);
            dump_memory(emu.mem.as_slice(), initial_memory.as_slice());
            println!("");
            emu = reset_emulator();
            parse_state = Description;
            continue;
        }

        match parse_state {
            Description => {
                parse_state = GeneralRegs;
                description = line;
            },
            GeneralRegs => {
                parse_state = ExtraRegs;
                parse_general_regs(&mut emu.cpu, line.as_slice());
            },
            ExtraRegs => {
                parse_state = Memory;
                end_tstates = parse_extra_regs(&mut emu.cpu, line.as_slice());
            },
            Memory => {
                parse_memory(emu.mem.as_mut_slice(), line.as_slice());
            }
        }
    }    
}
