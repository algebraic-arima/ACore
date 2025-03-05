use riscv::register::mstatus::*;
use riscv::register::sstatus::*;


#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Mstatus,
    pub sepc: usize,
}

// impl TrapContext {
//     pub fn os_init_context(entry: usize, sp: usize) -> Self {
//         let sstatus = sstatus::read();
//         unsafe {
//             mstatus::set_mpp(MPP::Supervisor);
//         }
//         let mut ctx = Self {
//             x: [0; 32],
//             sstatus: sstatus,
//             sepc: entry,
//         };
//         ctx.x[2] = sp;
//         ctx
//     }
// }
