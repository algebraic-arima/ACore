//! `Arc<Inode>` -> `OSInodeInner`: In order to open files concurrently
//! we need to wrap `Inode` into `Arc`,but `Mutex` in `Inode` prevents
//! file systems from being accessed simultaneously
//!
//! `UPSafeCell<OSInodeInner>` -> `OSInode`: for static `ROOT_INODE`,we
//! need to wrap `OSInodeInner` into `UPSafeCell`
use super::File;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use crate::{drivers::BLOCK_DEVICE, fs::inode};
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::*;
use fs::{FileSystem, Inode};
use lazy_static::*;
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            // println!("before read_at: offset = {}", inner.offset);
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            // println!("after read_at: offset = {}, len = {}", inner.offset, len);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let fs = FileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(FileSystem::root_inode(&fs))
    };
    // pub static ref BIN_INODE: Arc<Inode> = {
    //     ROOT_INODE.mkdir("bin").unwrap()
    // };
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    let bin_inode = ROOT_INODE
        .find("bin")
        .unwrap_or_else(|| ROOT_INODE.create("bin").expect("No /bin directory"));
    for app in bin_inode.ls() {
        println!("/bin/{}", app);
    }
    println!("**************/");
}

bitflags! {
    ///Open file flags
    pub struct OpenFlags: u32 {
        ///Read only
        const RDONLY = 0;
        ///Write only
        const WRONLY = 1 << 0;
        ///Read & Write
        const RDWR = 1 << 1;
        ///Allow create
        const CREATE = 1 << 9;
        ///Clear file and return an empty one
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}
///Open file with flags
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            // clear size
            // println!("\nopen: {}", name);
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            // println!("\nopen create: {}", name);
            ROOT_INODE
                .create(name)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        // println!("\nopen nocreate file: {}", name);
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

///Open file with flags
pub fn open_bin(name: &str) -> Option<Arc<OSInode>> {
    let (readable, writable) = OpenFlags::RDONLY.read_write();
    // println!("\nopen nocreate file: {}", name);
    ROOT_INODE
        .find_elf(name)
        .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
}

pub fn mkdir_at_root(name: &str) -> Option<Arc<OSInode>> {
    ROOT_INODE
        .mkdir(name)
        .map(|inode| Arc::new(OSInode::new(true, false, inode)))
}

pub fn remove_at_root(name: &str) -> bool {
    ROOT_INODE.remove(name)
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}
