use libc::{c_ushort, c_uchar, c_uint, c_int};
use std::mem::transmute;

pub type Byte = c_uchar;
pub struct UserData;

pub type Z80DataIn = extern "C" fn(
        param: c_int,
        addr: c_ushort,
        usr_data: *mut UserData
    ) -> Byte;

pub type Z80DataOut = extern "C" fn(
        param: c_int,
        addr: c_ushort, 
        data: Byte, 
        usr_data: *mut UserData
    );

#[repr(C)]
#[allow(uppercase_variables)]
#[allow(dead_code)]
pub struct Wr {
    pub AF: c_ushort,
    pub BC: c_ushort,
    pub DE: c_ushort,
    pub HL: c_ushort,
    pub IX: c_ushort,
    pub IY: c_ushort,
    pub SP: c_ushort,
}

#[repr(C)]
pub struct Z80Regs {
    pub data: [u16, ..7u],
}

impl Z80Regs {
    
    pub fn wr(&mut self) -> &mut Wr {
        unsafe { transmute(self) }
    }    
}


#[repr(C)]
#[allow(uppercase_variables)]
pub struct Z80Context {
    pub R1: Z80Regs,
    pub R2: Z80Regs,
    pub PC: c_ushort,
    pub R: Byte,
    pub I: Byte,
    pub IFF1: Byte,
    pub IFF2: Byte,
    pub IM: Byte,
    pub memRead: Option<Z80DataIn>,
    pub memWrite: Option<Z80DataOut>,
    pub memParam: c_int,
    pub ioRead: Option<Z80DataIn>,
    pub ioWrite: Option<Z80DataOut>,
    pub ioParam: c_int,
    pub halted: Byte,
    pub tstates: c_uint,
    pub nmi_req: Byte,
    pub int_req: Byte,
    pub defer_int: Byte,
    pub int_vector: Byte,
    pub exec_int_vector: Byte,
    pub user_data: *mut UserData
}

#[link(name = "z80")]
extern "C" {
    pub fn Z80Execute(ctx: *mut Z80Context);
    pub fn Z80ExecuteTStates(ctx: *mut Z80Context, tstates: c_uint) ->
     c_uint;

    pub fn Z80RESET(ctx: *mut Z80Context);
    pub fn Z80INT(ctx: *mut Z80Context, value: Byte);
    pub fn Z80NMI(ctx: *mut Z80Context);
}
