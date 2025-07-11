use core::alloc::Layout;
use std::slice::from_raw_parts;

#[repr(transparent)]
pub struct FatPointer(pub u64);

impl FatPointer {
    pub fn offset(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn size(&self) -> u32 {
        self.0 as u32
    }

    pub fn copy_data(&self) -> Box<[u8]> {
        let slice = unsafe {
            from_raw_parts(self.offset() as *const u8, self.size() as usize)
        };
        Box::from(slice)
    }
}

pub fn write_to_host(value: &[u8]) -> FatPointer {
    return leak_to_shared_memory(value)
}

// will panic if value is too big
// takes byte slice to leak
// returns u64 fat pointer to location in memory
pub fn leak_to_shared_memory(value: &[u8]) -> FatPointer {
    let layout = Layout::array::<u8>(value.len()).unwrap();
    unsafe {
        let new_ptr = std::alloc::alloc(layout);
        core::ptr::copy(value.as_ptr(), new_ptr, value.len());
        FatPointer((new_ptr as u64) << 32u64 | value.len() as u64)
    }
}
