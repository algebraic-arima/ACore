use clap::{App, Arg};
use fs::{BlockDevice, FileSystem};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

const BLOCK_SZ: usize = 512;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}

fn main() {
    fs_pack().expect("Error when packing fs!");
}

fn fs_pack() -> std::io::Result<()> {
    let matches = App::new("FileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path = {}", src_path, target_path);
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}", target_path, "fs.img"))?;
        f.set_len(16 * 2048 * 512).unwrap();
        f
    })));
    // 16MiB, at most 4095 files
    let fs = FileSystem::create(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(FileSystem::root_inode(&fs));
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    for app in apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in fs
        let inode = root_inode.create(app.as_str()).unwrap();
        // write data to fs
        inode.write_at(0, all_data.as_slice());
        // let i = inode.read_at(0, &mut [0u8; 1024]);
        // println!("Test Read {} bytes from app file: {}", i, app);
        println!("Created app file: {}, bytes: {}", app, all_data.len());
    }
    // list apps
    // for app in root_inode.ls() {
    //     println!("{}", app);
    // }
    Ok(())
}

#[test]
fn fs_test() -> std::io::Result<()> {
    print!("Loading img...\n");
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len(8192 * 512).unwrap();
        f
    })));
    FileSystem::create(block_file.clone(), 4096, 1);
    let fs = FileSystem::open(block_file.clone());
    println!("Testing creating files/dirs in root...");
    let root_inode = FileSystem::root_inode(&fs);
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    root_inode.create("filea");
    root_inode.create("fileb");
    root_inode.mkdir("usr");
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    println!("Testing removing fileb...");
    root_inode.remove("fileb");
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    let null_inode = root_inode.find("fileb");
    assert!(null_inode.is_none(), "fileb should be removed!");
    println!("Testing creating filec in /usr...");
    let usr_inode = root_inode.find("usr").unwrap();
    usr_inode.create("filec");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    let par_node = usr_inode.find("..").unwrap();
    println!("Testing removing directory tmp...");
    root_inode.mkdir("tmp");
    let tmp_inode = root_inode.find("tmp").unwrap();
    tmp_inode.create("file_in_tmp");
    for name in tmp_inode.ls() {
        println!("/tmp: {}", name);
    }
    let file_in_tmp = tmp_inode.find("file_in_tmp").unwrap();
    file_in_tmp.write_at(0, b"Hello, tmp!");
    let mut buffer = [0u8; 512];
    let len = file_in_tmp.read_at(0, &mut buffer);
    assert_eq!(
        core::str::from_utf8(&buffer[..len]).unwrap(),
        "Hello, tmp!",
        "Read content should match!"
    );
    println!("Content of file_in_tmp: {}", core::str::from_utf8(&buffer[..len]).unwrap());
    root_inode.remove("tmp");
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    assert!(root_inode.find("tmp").is_none(), "tmp should be removed!");

    println!("Testing renaming...");
    usr_inode.rename("filec", "filed");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(usr_inode.find("filec").is_none(), "filec should be renamed to filed!");
    assert!(usr_inode.find("filed").is_some(), "filed should exist!");
    root_inode.rename("usr", "user");
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    assert!(root_inode.find("usr").is_none(), "usr should be renamed to user!");
    assert!(root_inode.find("user").is_some(), "user should exist!");
    root_inode.rename("user", "usr");

    // println!("Testing absolute path finding...");
    // let filed_inode = usr_inode.find("filed").unwrap();
    // let filed_inode_tmp = FileSystem::abs_path_to_inode(&fs, "/usr/filed").unwrap();
    // assert!(filed_inode_tmp.is_file(), "filed should be a file!");
    // assert!(filed_inode_tmp.get_block_id() == filed_inode.get_block_id(), "filed inode should match!");
    // assert!(filed_inode_tmp.get_block_offset() == filed_inode.get_block_offset(), "filed inode offset should match!");

    println!("Testing writing and reading filea...");
    let filea = par_node.find("filea").unwrap();
    let greet_str = "Hello, world!";
    filea.write_at(0, greet_str.as_bytes());
    let mut buffer = [0u8; 512];
    // let mut buffer = [0u8; 23];
    let len = filea.read_at(0, &mut buffer);
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

    let mut random_str_test = |len: usize| {
        filea.clear();
        assert_eq!(filea.read_at(0, &mut buffer), 0,);
        let mut str = String::new();
        use rand;
        // random digit
        for _ in 0..len {
            str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
        }
        filea.write_at(0, str.as_bytes());
        let mut read_buffer = [0u8; 127];
        let mut offset = 0usize;
        let mut read_str = String::new();
        loop {
            let len = filea.read_at(offset, &mut read_buffer);
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
        }
        assert_eq!(str, read_str);
    };

    random_str_test(4 * BLOCK_SZ);
    random_str_test(8 * BLOCK_SZ + BLOCK_SZ / 2);
    random_str_test(100 * BLOCK_SZ);
    random_str_test(70 * BLOCK_SZ + BLOCK_SZ / 7);
    random_str_test((12 + 128) * BLOCK_SZ);
    random_str_test(400 * BLOCK_SZ);
    random_str_test(1000 * BLOCK_SZ);
    random_str_test(2000 * BLOCK_SZ);

    Ok(())
}
