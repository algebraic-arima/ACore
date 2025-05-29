use core::any::Any;
/// A trait for block devices that can read and write blocks of data.
pub trait BlockDevice : Send + Sync + Any {
    /// Read from block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
}