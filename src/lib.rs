#![crate_name = "z80"]
#![crate_type = "rlib"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate libc;

use libc::{c_ushort, c_uchar, c_uint, c_int};
use std::mem::transmute;

pub struct UserData;

pub type Z80DataIn = extern "C" fn(
        param: c_int,
        addr: c_ushort,
        usr_data: *mut UserData
    ) -> c_uchar;

pub type Z80DataOut = extern "C" fn(
        param: c_int,
        addr: c_ushort, 
        data: c_uchar, 
        usr_data: *mut UserData
    );

#[repr(C)]
#[allow(uppercase_variables)]
#[allow(dead_code)]
pub struct WordRegisters {
    pub AF: c_ushort,
    pub BC: c_ushort,
    pub DE: c_ushort,
    pub HL: c_ushort,
    pub IX: c_ushort,
    pub IY: c_ushort,
    pub SP: c_ushort,
}

#[repr(C)]
pub struct Registers {
    pub data: [u16, ..7u],
}

impl Registers {
    
    pub fn word_registers(&mut self) -> &mut WordRegisters {
        unsafe { transmute(self) }
    }    
}


#[repr(C)]
#[allow(uppercase_variables)]
pub struct Context {
    pub R1: Registers,
    pub R2: Registers,
    pub PC: c_ushort,
    pub R: c_uchar,
    pub I: c_uchar,
    pub IFF1: c_uchar,
    pub IFF2: c_uchar,
    pub IM: c_uchar,
    pub mem_read: Option<Z80DataIn>,
    pub mem_write: Option<Z80DataOut>,
    pub mem_param: c_int,
    pub io_read: Option<Z80DataIn>,
    pub io_write: Option<Z80DataOut>,
    pub io_param: c_int,
    pub halted: c_uchar,
    pub tstates: c_uint,
    pub nmi_req: c_uchar,
    pub int_req: c_uchar,
    pub defer_int: c_uchar,
    pub int_vector: c_uchar,
    pub exec_int_vector: c_uchar,
    pub user_data: *mut UserData
}

macro_rules! reg_access(
    ($write_fn:ident, $read_fn:ident, $reg:ident, $reg_set:ident, $reg_ty:ty) =>
    (
        impl Context {
            #[allow(non_snake_case_functions)]
            pub fn $write_fn(&mut self, val: $reg_ty) {
                self.$reg_set.word_registers().$reg = val;
            }

            #[allow(non_snake_case_functions)]
            pub fn $read_fn(&mut self) -> $reg_ty {
                self.$reg_set.word_registers().$reg
            }
        }
    );
)

reg_access!(set_AF1, get_AF1, AF, R1, u16)
reg_access!(set_BC1, get_BC1, BC, R1, u16)
reg_access!(set_DE1, get_DE1, DE, R1, u16)
reg_access!(set_HL1, get_HL1, HL, R1, u16)
reg_access!(set_AF2, get_AF2, AF, R2, u16)
reg_access!(set_BC2, get_BC2, BC, R2, u16)
reg_access!(set_DE2, get_DE2, DE, R2, u16)
reg_access!(set_HL2, get_HL2, HL, R2, u16)
reg_access!(set_IY,  get_IY,  IY, R1, u16)
reg_access!(set_IX,  get_IX,  IX, R1, u16)
reg_access!(set_SP,  get_SP,  SP, R1, u16)

impl Context {

    pub fn new() -> Context {
        let z80_context = Context {
            R1: Registers { data: [0, ..7u] },
            R2: Registers { data: [0, ..7u] },
            PC: 0, R: 0, I: 0, 
            IFF1: 0, IFF2: 0, IM: 0,
            mem_read: None, mem_write: None,
            mem_param: 0,
            io_read: None, io_write: None,
            io_param: 0, halted: 0,
            tstates: 0,
            nmi_req: 0, int_req: 0,
            defer_int: 0, int_vector: 0,
            exec_int_vector: 0,
            user_data: std::ptr::mut_null()
        };

        unsafe { Z80RESET( transmute(&z80_context) ) };

        z80_context
    }

    pub fn execute(&mut self) -> u32 {
        let tstates_pre = self.tstates;
        unsafe {
            Z80Execute( transmute(&self) );
        }
        let tstates_post = self.tstates;
        tstates_post - tstates_pre
    }

    pub fn set_irq_line(&mut self, high: bool) {
        if high {
            self.int_req = 1;
        } else {
            self.int_req = 0;
        }
    }

    pub fn is_halted(&self) -> bool {
        self.halted > 1u8
    }

    pub fn non_maskable_interrupt(&mut self) {
        unsafe {
            Z80NMI(transmute(self));
        }
    }

    pub fn interrupt(&mut self, bus_val: u8) {
        unsafe {
            Z80INT(transmute(self), bus_val);
        }
    }

    pub fn execute_tstates(&mut self, cycles: u32) -> u32 {
        unsafe {
            Z80ExecuteTStates(transmute(self), cycles)
        }
    }

}


#[link(name = "z80", kind = "static")]
extern "C" {
    fn Z80Execute(ctx: *mut Context);
    fn Z80ExecuteTStates(ctx: *mut Context, tstates: c_uint) ->
     c_uint;

    fn Z80RESET(ctx: *mut Context);
    fn Z80INT(ctx: *mut Context, value: c_uchar);
    fn Z80NMI(ctx: *mut Context);
}
