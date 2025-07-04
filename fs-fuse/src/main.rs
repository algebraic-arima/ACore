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
    root_inode.mkdir("bin");
    let bin_inode = root_inode.find("bin").unwrap();
    for app in apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in fs
        let inode = bin_inode.create(app.as_str()).unwrap();
        // write data to fs
        inode.write_at(0, all_data.as_slice());
        // let i = inode.read_at(0, &mut [0u8; 1024]);
        // println!("Test Read {} bytes from app file: {}", i, app);
        println!("Created app file: {}, bytes: {}", app, all_data.len());
    }
    // list apps
    // for app in bin_inode.ls() {
    //     println!("/bin: {}", app);
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
    let fileb_inode = root_inode.find("fileb").unwrap();
    fileb_inode.write_at(0, b"Hello, fileb!");
    let mut buffer = [0u8; 512];
    root_inode.remove("fileb");
    let len = fileb_inode.read_at(0, &mut buffer);
    assert_eq!(len, 13, "fileb data remains!");
    assert!(root_inode.find("fileb").is_none(), "fileb should not exist!");

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

    root_inode.rename("usr/filed", "filec");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(usr_inode.find("filed").is_none(), "filed should be renamed to filed!");
    assert!(usr_inode.find("filec").is_some(), "filec should exist!");

    println!("Testing relative path find...");
    let venillalemon_inode = usr_inode.mkdir("venillalemon").unwrap();
    let yuchuan_none_inode = usr_inode.mkdir("usr/yuchuan");
    assert!(yuchuan_none_inode.is_none(), "/usr/usr/yuchuan should not exist!");
    root_inode.mkdir("usr/yuchuan");
    let yuchuan_inode = usr_inode.find("yuchuan");
    assert!(yuchuan_inode.is_some(), "/usr/yuchuan should exist!");
    let yuchuan_inode = yuchuan_inode.unwrap();
    usr_inode.mkdir("Czar");
    let modist_inode = root_inode.mkdir("usr/modist").unwrap();
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    let venillalemon_test_inode = usr_inode.find("venillalemon");
    assert!(venillalemon_test_inode.is_some(), "/usr/venillalemon should exist!");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    for name in venillalemon_inode.ls() {
        println!("/usr/venillalemon: {}", name);
    }
    let test_usr_inode = venillalemon_inode.find("../../usr").unwrap();
    for name in test_usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(test_usr_inode.find("filec").is_some(), "filec should exist in /usr!");
    assert!(test_usr_inode.find("venillalemon").is_some(), "venillalemon should exist in /usr!");

    println!("Testing moving usr/modist to usr/yuchuan/modist...");
    usr_inode.mv("modist", "yuchuan");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(usr_inode.find("modist").is_none(), "modist should be moved to /usr/yuchuan/modist!");
    assert!(usr_inode.find("yuchuan/modist").is_some(), "modist should exist in /usr/yuchuan/modist!");
    for name in yuchuan_inode.ls() {
        println!("/usr/yuchuan: {}", name);
    }
    let usr_yuchuan_modist_inode = yuchuan_inode.find("modist").unwrap();
    root_inode.mv("usr/filec", "usr/yuchuan/modist");
    for name in usr_yuchuan_modist_inode.ls() {
        println!("/usr/yuchuan/modist: {}", name);
    }
    assert!(usr_yuchuan_modist_inode.find("filec").is_some(), "filec should exist in /usr/yuchuan/modist!");
    assert!(usr_inode.find("filec").is_none(), "filec should be removed from /usr!");
    root_inode.mv("usr/yuchuan/modist", "usr");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(usr_inode.find("yuchuan/modist").is_none(), "modist should be moved to /usr/modist!");
    assert!(usr_inode.find("modist").is_some(), "modist should exist in /usr/modist!");
    for name in usr_inode.find("modist").unwrap().ls() {
        println!("/usr/modist: {}", name);
    }

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

    println!("Testing removing usr...");
    usr_inode.create("profile");
    println!("After creating profile:");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    let tmp = usr_inode.find("profile");
    assert!(tmp.is_some(), "profile should exist in /usr!");
    let profile_inode = tmp.unwrap();
    profile_inode.write_at(0, b"Hello, profile!");
    let mut buffer = [0u8; 512];
    let len = profile_inode.read_at(0, &mut buffer);
    assert_eq!(
        core::str::from_utf8(&buffer[..len]).unwrap(),
        "Hello, profile!",
        "Read content should match!"
    );
    let be = root_inode.remove("usr/modist");
    println!("After removing usr/modist under root...");
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(be, "Removing usr/modist should succeed!");
    let b = root_inode.remove("usr");
    assert!(b, "Removing usr should succeed!");
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    for name in usr_inode.ls() {
        println!("/usr: {}", name);
    }
    assert!(root_inode.find("usr").is_none(), "usr should be removed!");
    assert!(root_inode.find("usr/profile").is_none(), "profile should be removed with usr!");
    let len = profile_inode.read_at(0, &mut buffer);
    assert_eq!(len, 15, "profile data remains!");

    let test_str = "Hello, filea in tmp!";
    let tmp_name = "tmp";
    let tmp_inode = root_inode.mkdir(tmp_name).unwrap();
    let path = "tmp/filea";
    let fa = root_inode.create(path).unwrap();
    fa.write_at(0, test_str.as_bytes());
    let mut buffer = [0u8; 100];
    let len = fa.read_at(0, &mut buffer);
    assert_eq!(
        core::str::from_utf8(&buffer[..len]).unwrap(),
        "Hello, filea in tmp!",
        "Read content should match!"
    );
    for name in root_inode.ls() {
        println!("/: {}", name);
    }
    let tmp_tmp_inode = root_inode.find("tmp").unwrap();
    for name in tmp_inode.ls() {
        println!("/tmp: {}", name);
    }

    println!("All tests passed!");

    Ok(())
}
