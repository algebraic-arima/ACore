//!An easy file system isolated from the kernel
extern crate alloc;
extern crate lru;
mod bitmap;
mod block_cache;
mod block_dev;
mod fs;
mod layout;
mod vfs;
/// Use a block size of 512 bytes, directory entry size of 32 bytes
pub const DIRENT_SZ: usize = 32;
pub const BLOCK_SZ: usize = 512;
type DataBlock = [u8; BLOCK_SZ];

use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use fs::FileSystem;
use layout::*;
pub use vfs::Inode;
