use log::info;


#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    unsafe extern "C" {
        safe fn start_bss();
        safe fn end_bss();
    }
    let bss_range = start_bss as usize..end_bss as usize;
    info!("bss_range: {:x?}..{:x?}", bss_range.start, bss_range.end);
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    info!("heap_test passed!");
}
