use core::alloc::Layout;

//mod utils;

// will panic if value is too big
// takes byte slice to leak
// returns u64 fat pointer to location in memory
pub fn leak_to_shared_memory(value: &[u8]) -> u64 {
    unsafe {
        let layout = Layout::array::<u8>(value.len()).unwrap();
        let new_ptr = std::alloc::alloc(layout);
        core::ptr::copy(value.as_ptr(), new_ptr, value.len());
        return ((new_ptr as u64) << 32u64) | value.len() as u64
    }
}

