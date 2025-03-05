use riscv::register::mstatus::{self, MPP, Mstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub mstatus: Mstatus,
    pub mepc: usize,
}

impl TrapContext {
    pub fn os_init_context(entry: usize, sp: usize) -> Self {
        let mstatus = mstatus::read();
        unsafe {
            mstatus::set_mpp(MPP::Supervisor);
        }
        let mut ctx = Self {
            x: [0; 32],
            mstatus: mstatus,
            mepc: entry,
        };
        ctx.x[2] = sp;
        ctx
    }
}
