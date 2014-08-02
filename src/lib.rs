#![crate_name = "z80"]
#![crate_type = "rlib"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate libc;

use std::mem::transmute;
pub use ffi::{Z80DataIn, Z80DataOut, UserData};
pub mod ffi;


pub struct Context {
    ctx: ffi::Z80Context,
}


macro_rules! reg_access(
    ($write_fn:ident, $read_fn:ident, $reg:ident, $reg_set:ident, $reg_ty:ty) =>
    (
        impl Context {
            #[allow(non_snake_case_functions)]
            pub fn $write_fn(&mut self, val: $reg_ty) {
                self.ctx.$reg_set.wr().$reg = val;
            }

            #[allow(non_snake_case_functions)]
            pub fn $read_fn(&mut self) -> $reg_ty {
                self.ctx.$reg_set.wr().$reg
            }
        }
    );

    ($write_fn:ident, $read_fn:ident, $reg:ident, $reg_ty:ty) =>
    (
        impl Context {
            #[allow(non_snake_case_functions)]
            pub fn $write_fn(&mut self, val: $reg_ty) {
                self.ctx.$reg = val;
            }

            #[allow(non_snake_case_functions)]
            pub fn $read_fn(&self) -> $reg_ty {
                self.ctx.$reg
            }
        }
    );
)

reg_access!(write_AF1, read_AF1, AF, R1, u16)
reg_access!(write_BC1, read_BC1, BC, R1, u16)
reg_access!(write_DE1, read_DE1, DE, R1, u16)
reg_access!(write_HL1, read_HL1, HL, R1, u16)
reg_access!(write_AF2, read_AF2, AF, R2, u16)
reg_access!(write_BC2, read_BC2, BC, R2, u16)
reg_access!(write_DE2, read_DE2, DE, R2, u16)
reg_access!(write_HL2, read_HL2, HL, R2, u16)
reg_access!(write_IY,  read_IY,  IY, R1, u16)
reg_access!(write_IX,  read_IX,  IX, R1, u16)
reg_access!(write_SP,  read_SP,  SP, R1, u16)

reg_access!(write_halted,  read_halted,  halted,  u8)
reg_access!(write_I,       read_I,       I,       u8)
reg_access!(write_IM,      read_IM,      IM,      u8)
reg_access!(write_IFF1,    read_IFF1,    IFF1,    u8)
reg_access!(write_IFF2,    read_IFF2,    IFF2,    u8)
reg_access!(write_R,       read_R,       R,       u8)
reg_access!(write_PC,      read_PC,      PC,      u16)
reg_access!(write_tstates, read_tstates, tstates, u32)

impl Context {

    pub fn new() -> Context {
        let z80_context = ffi::Z80Context {
            R1: ffi::Z80Regs { data: [0, ..7u] },
            R2: ffi::Z80Regs { data: [0, ..7u] },
            PC: 0, R: 0, I: 0, 
            IFF1: 0, IFF2: 0, IM: 0,
            memRead: None, memWrite: None,
            memParam: 0,
            ioRead: None, ioWrite: None,
            ioParam: 0, halted: 0,
            tstates: 0,
            nmi_req: 0, int_req: 0,
            defer_int: 0, int_vector: 0,
            exec_int_vector: 0,
            user_data: std::ptr::mut_null()
        };

        unsafe { ffi::Z80RESET( transmute(&z80_context) ) };

        Context { 
            ctx: z80_context
        }
    }

    pub fn execute(&mut self) {
        unsafe {
            ffi::Z80Execute( transmute(&self.ctx) );
        }
    }

    pub fn is_halted(&self) -> bool {
        self.ctx.halted > 1u8
    }

    pub fn non_maskable_interrupt(&mut self) {
        unsafe {
            ffi::Z80NMI(transmute(self));
        }
    }

    pub fn clear_interrupt(&mut self) {
        self.ctx.int_vector = 0;
    }

    pub fn interrupt(&mut self, bus_val: u8) {
        unsafe {
            ffi::Z80INT(transmute(self), bus_val);
        }
    }

    pub fn execute_tstates(&mut self, cycles: u32) -> u32 {
        unsafe {
            ffi::Z80ExecuteTStates(transmute(self), cycles)
        }
    }

    pub fn set_io_read_callback(&mut self, cb: Option<Z80DataIn>) {
        self.ctx.ioRead = cb;
    }

    pub fn set_io_write_callback(&mut self, cb: Option<Z80DataOut>) {
        self.ctx.ioWrite = cb;
    }

    pub fn set_mem_read_callback(&mut self, cb: Option<Z80DataIn>) {
        self.ctx.memRead = cb;
    }

    pub fn set_mem_write_callback(&mut self, cb: Option<Z80DataOut>) {
        self.ctx.memWrite = cb;
    }

    pub fn set_memory_accessor(&mut self, accessor: *mut UserData) {
        self.ctx.user_data = accessor;
    }



}
