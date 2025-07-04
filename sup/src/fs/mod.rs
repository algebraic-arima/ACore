//! File system in os
mod inode;
mod stdio;
mod pipe;

use crate::mm::UserBuffer;
/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use inode::{OSInode, OpenFlags, list_apps, open_file, open_bin, mkdir_at_root, remove_at_root, rename_at_root, move_at_root};
pub use pipe::{make_pipe, Pipe, PipeRingBuffer};
pub use stdio::{Stdin, Stdout};
