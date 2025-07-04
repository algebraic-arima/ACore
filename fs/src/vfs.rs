use super::{
    BlockDevice, DIRENT_SZ, DirEntry, DiskInode, DiskInodeType, FileSystem, block_cache_sync_all,
    get_block_cache,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};

pub struct Inode {
    block_id: usize,     // block id on disk
    block_offset: usize, // dist offset in block
    fs: Arc<Mutex<FileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    pub fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }

    pub fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }

    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<FileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }

    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

    fn remove_inode_id(&self, name: &str, disk_inode: &mut DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        // let mut flag = false;
        // let mut inode_id = None;
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            // if flag {
            //     disk_inode.write_at(
            //         (i - 1) * DIRENT_SZ,
            //         dirent.as_bytes(),
            //         &self.block_device,
            //     );
            //     continue;
            // }
            if dirent.name() == name {
                // remove the dirent
                disk_inode.write_at(DIRENT_SZ * i, &[0; DIRENT_SZ], &self.block_device);
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

    fn _find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode: &DiskInode| {
            assert!(disk_inode.is_dir());
            let inode_id = self.find_inode_id(name, disk_inode);
            if let Some(inode_id) = inode_id {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Some(Arc::new(Self::new(
                    block_id,
                    block_offset,
                    Arc::clone(&self.fs),
                    Arc::clone(&self.block_device),
                )))
            } else {
                None
            }
        })
    }

    pub fn find_elf(&self, name: &str) -> Option<Arc<Inode>> {
        // find a file in bin folder of current directory
        if name.is_empty() {
            return None;
        }
        let elf_inode = self._find("bin")?;
        if !elf_inode.read_disk_inode(|disk_inode| disk_inode.is_dir()) {
            return None; // bin is not a directory
        }
        elf_inode._find(name)
    }

    pub fn find(&self, path: &str) -> Option<Arc<Inode>> {
        // find a file or directory in current directory, relative
        if path.is_empty() {
            return None;
        }
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        for name in path.split('/') {
            if name.is_empty() {
                return None; // invalid absolute path
            }
            if let Some(next_inode) = cur_inode._find(name) {
                cur_inode = next_inode;
            } else {
                return None; // not found
            }
        }
        Some(cur_inode)
    }

    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                // if name is empty, skip
                if dirent.is_empty() {
                    continue;
                }
                v.push(String::from(dirent.name()));
            }
            v
        })
    }

    /// create a new file, but every dir should exist before the last one
    pub fn create(&self, path: &str) -> Option<Arc<Inode>> {
        // create a new file, but every dir should exist before the last one
        if path.is_empty() {
            return None;
        }
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let len = path.split('/').count();
        for (i, name) in path.split('/').enumerate() {
            if name.is_empty() {
                return None; // invalid absolute path
            }
            if i == len - 1 {
                return cur_inode._create(name); // create the new file
            } else {
                if let Some(next_inode) = cur_inode._find(name) {
                    cur_inode = next_inode;
                } else {
                    return None; // dir not found
                }
            }
        }
        None
    }

    fn _create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |cur_dir_inode: &DiskInode| {
            // assert it is a directory
            assert!(cur_dir_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, cur_dir_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|cur_dir_inode| {
            // append file in the dirent
            let file_count = (cur_dir_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, cur_dir_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            cur_dir_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release fs lock automatically by compiler
    }

    pub fn mkdir(&self, path: &str) -> Option<Arc<Inode>> {
        // create a new dir, but every dir should exist before the last one
        if path.is_empty() {
            return None;
        }
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let len = path.split('/').count();
        for (i, name) in path.split('/').enumerate() {
            if name.is_empty() {
                return None; // invalid absolute path
            }
            if i == len - 1 {
                if let Some(new_inode) = cur_inode._mkdir(name) {
                    return Some(new_inode); // create the new directory
                } else {
                    return None; // dir not created
                }
            } else {
                if let Some(next_inode) = cur_inode._find(name) {
                    cur_inode = next_inode;
                } else {
                    return None; // dir not found
                }
            }
        }
        Some(cur_inode)
    }

    /// Create a new directory
    fn _mkdir(&self, name: &str) -> Option<Arc<Inode>> {
        if name.is_empty() {
            return None; // invalid directory name
        }
        let mut fs = self.fs.lock();
        let op = |cur_dir_inode: &DiskInode| {
            // assert it is a directory
            assert!(cur_dir_inode.is_dir());
            // has the directory been created?
            self.find_inode_id(name, cur_dir_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }

        // create a new directory
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();

        // write the inode_id to the parent directory
        self.modify_disk_inode(|cur_dir_inode| {
            // append file in the dirent
            let file_count = (cur_dir_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, cur_dir_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            cur_dir_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        // initialize new inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::Directory);
            });
        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        // return inode
        let new_inode = Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        ));
        // initialize . and .. in directory
        new_inode.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.is_dir());
            let new_size = 2 * DIRENT_SZ; // . and ..
            // increase size
            self.increase_size(new_size as u32, disk_inode, &mut fs); // no matter what Inode calls increase
            // . -> self
            let dirent = DirEntry::new(".", new_inode_id);
            disk_inode.write_at(0, dirent.as_bytes(), &self.block_device);
            // .. -> parent
            let parent_inode_id = fs.get_inode_id(self.block_id as u32, self.block_offset);
            let dirent = DirEntry::new("..", parent_inode_id);
            disk_inode.write_at(DIRENT_SZ, dirent.as_bytes(), &self.block_device);
        });
        block_cache_sync_all();
        Some(new_inode)
    }

    /// remove a file or directory, relative path
    pub fn remove(&self, path: &str) -> bool {
        // remove a file or directory in current directory, relative
        if path.is_empty() {
            return false;
        }
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let len = path.split('/').count();
        for (i, name) in path.split('/').enumerate() {
            if name.is_empty() {
                return false; // invalid absolute path
            }
            if i == len - 1 {
                return cur_inode._remove(name); // remove the file or directory
            } else {
                if let Some(next_inode) = cur_inode._find(name) {
                    cur_inode = next_inode;
                } else {
                    return false; // dir not found
                }
            }
        }
        false
    }

    /// remove a file or directory, single find
    fn _remove(&self, name: &str) -> bool {
        // remove a file or directory
        let mut fs = self.fs.lock();
        self.__remove(name, &mut fs)
    }

    /// remove a file or directory, recursively
    fn __remove(&self, name: &str, fs: &mut MutexGuard<'_, FileSystem>) -> bool {
        let mut inode_id = None;
        let mut disk_inode = None;
        self.modify_disk_inode(|cur_dir_inode| {
            // assert it is a directory
            assert!(cur_dir_inode.is_dir());
            inode_id = self.remove_inode_id(name, cur_dir_inode);
        });
        if let Some(inode_id) = inode_id {
            fs.dealloc_inode(inode_id);
            // dealloc data blocks
            let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
            get_block_cache(block_id as usize, Arc::clone(&self.block_device))
                .lock()
                .modify(block_offset, |dinode: &mut DiskInode| {
                    disk_inode = Some(dinode.clone());
                });
            // disk_inode is the inode to be removed
            if let Some(mut disk_inode) = disk_inode {
                if disk_inode.is_dir() {
                    // recursively remove all files in the directory
                    let file_count = (disk_inode.size as usize) / DIRENT_SZ;
                    // then construct the Inode
                    let dir_inode = Arc::new(Self::new(
                        block_id,
                        block_offset,
                        self.fs.clone(),
                        self.block_device.clone(),
                    ));
                    // the first two entries are . and ..
                    assert!(file_count >= 2);
                    for i in 2..file_count {
                        let mut dirent = DirEntry::empty();
                        assert_eq!(
                            disk_inode.read_at(
                                i * DIRENT_SZ,
                                dirent.as_bytes_mut(),
                                &self.block_device,
                            ),
                            DIRENT_SZ,
                        );
                        if dirent.is_empty() {
                            continue;
                        }
                        dir_inode.__remove(dirent.name(), fs);
                    }
                }
                // dealloc data blocks possessed by the file/dir inode
                let size = disk_inode.size;
                let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
                assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
                for data_block in data_blocks_dealloc.into_iter() {
                    fs.dealloc_data(data_block);
                }
            }
            block_cache_sync_all();
            true
        } else {
            false
        }
    }

    pub fn rename(&self, path: &str, new_name: &str) -> bool {
        // rename a file or directory in current directory, relative
        if path.is_empty() || new_name.is_empty() {
            return false; // invalid name
        }
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let len = path.split('/').count();
        for (i, name) in path.split('/').enumerate() {
            if name.is_empty() {
                return false; // invalid absolute path
            }
            if i == len - 1 {
                return cur_inode._rename(name, new_name); // remove the file or directory
            } else {
                if let Some(next_inode) = cur_inode._find(name) {
                    cur_inode = next_inode;
                } else {
                    return false; // dir not found
                }
            }
        }
        false
    }

    /// rename a file or directory, relative path
    pub fn _rename(&self, old_name: &str, new_name: &str) -> bool {
        if old_name.is_empty() || new_name.is_empty() {
            return false; // invalid name
        }
        if old_name == new_name {
            return true; // no need to rename
        }
        let _fs = self.fs.lock();
        if self
            .read_disk_inode(|cur_dir_inode: &DiskInode| {
                // assert it is a directory
                assert!(cur_dir_inode.is_dir());
                // has the file been created?
                self.find_inode_id(new_name, cur_dir_inode)
            })
            .is_some()
        {
            return false;
        }

        let mut flag = false;
        self.modify_disk_inode(|disk_inode| {
            // find the old name
            let mut dirent = DirEntry::empty();
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            for i in 0..file_count {
                assert_eq!(
                    disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                if dirent.name() == old_name {
                    let new_dirent = DirEntry::new(new_name, dirent.inode_number());
                    disk_inode.write_at(DIRENT_SZ * i, new_dirent.as_bytes(), &self.block_device);
                    flag = true;
                }
            }
        });
        block_cache_sync_all();
        flag
    }

    pub fn mv(&self, path: &str, new_path: &str) -> bool {
        // move a file or directory in current directory, relative
        if path.is_empty() || new_path.is_empty() {
            return false; // invalid name
        }
        if path == new_path {
            return true; // no need to move
        }
        let mut dst_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let dst_len = new_path.split('/').count();
        if dst_len == 0 {
            return false; // invalid new path
        }
        for (i, name) in new_path.split('/').enumerate() {
            if name.is_empty() {
                return false; // invalid absolute path
            }
            if let Some(next_inode) = dst_inode._find(name) {
                dst_inode = next_inode;
            } else {
                return false; // dir not found
            }
        }
        assert!(dst_inode.read_disk_inode(|disk_inode| disk_inode.is_dir()));
        let mut cur_inode = Arc::new(Self::new(
            self.block_id as u32,
            self.block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.block_device),
        ));
        let len = path.split('/').count();
        for (i, name) in path.split('/').enumerate() {
            if name.is_empty() {
                return false; // invalid absolute path
            }
            if i == len - 1 {
                return cur_inode._move(name, dst_inode);
            } else {
                if let Some(next_inode) = cur_inode._find(name) {
                    cur_inode = next_inode;
                } else {
                    return false;
                }
            }
        }
        false
    }

    pub fn _move(&self, name: &str, dst_inode: Arc<Inode>) -> bool {
        // move a file or directory to dst_inode
        if name.is_empty() {
            return false; // invalid name
        }
        let mut fs = self.fs.lock();
        let mut inode_id = None;
        self.modify_disk_inode(|disk_inode| {
            // assert it is a directory
            assert!(disk_inode.is_dir());
            inode_id = self.remove_inode_id(name, disk_inode);
        });
        if let Some(inode_id) = inode_id {
            // write the inode_id to the destination directory
            dst_inode.modify_disk_inode(|dst_disk_inode| {
                assert!(dst_disk_inode.is_dir());
                let file_count = (dst_disk_inode.size as usize) / DIRENT_SZ;
                let new_size = (file_count + 1) * DIRENT_SZ;
                self.increase_size(new_size as u32, dst_disk_inode, &mut fs);
                let dirent = DirEntry::new(name, inode_id);
                dst_disk_inode.write_at(
                    file_count * DIRENT_SZ,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            });
            block_cache_sync_all();
            true
        } else {
            false
        }
    }

    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            // assert!(disk_inode.is_file());
            let len = disk_inode.read_at(offset, buf, &self.block_device);
            assert!(len <= buf.len());
            len
        })
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.is_file());
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }

    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<FileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
}
