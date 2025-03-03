use riscv::register::mstatus::{self, MPP, Mstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub mstatus: Mstatus,
    pub mepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn os_init_context(entry: usize, sp: usize) -> Self {
        let mstatus = mstatus::read(); // CSR sstatus
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
