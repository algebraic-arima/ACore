use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use super::{address::*, frame_allocator::*};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        !(self.flags() & PTEFlags::V).is_empty()
    }
    pub fn readable(&self) -> bool {
        !(self.flags() & PTEFlags::R).is_empty()
    }
    pub fn writable(&self) -> bool {
        !(self.flags() & PTEFlags::W).is_empty()
    }
    pub fn executable(&self) -> bool {
        !(self.flags() & PTEFlags::X).is_empty()
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    pub fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root_ppn;
        let idxs = vpn.indexes();
        for i in 0..2 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if pte.is_valid() {
                ppn = pte.ppn();
            } else {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                *pte = PageTableEntry::new(ppn, PTEFlags::V);
                self.frames.push(frame);
            }
        }
        let result = Some(&mut ppn.get_pte_array()[idxs[2]]);
        result
    }
    
    pub fn find_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root_ppn;
        let idxs = vpn.indexes();
        for i in 0..2 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if pte.is_valid() {
                ppn = pte.ppn();
            } else {
                return None;
            }
        }
        let result = Some(&mut ppn.get_pte_array()[idxs[2]]);
        result
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn.0);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is not mapped before unmapping", vpn.0);
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&mut self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    
    pub fn translate_va(&mut self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            //println!("translate_va:va = {:?}", va);
            let aligned_pa: PhysAddr = pte.ppn().into();
            //println!("translate_va:pa_align = {:?}", aligned_pa);
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }

    // generate legal satp
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
