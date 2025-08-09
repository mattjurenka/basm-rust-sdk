use core::alloc::Layout;
use std::slice::from_raw_parts;

/// Struct that represents a fat pointer where the upper 32 bits are the offset
/// and the lower 32 bits are the size of the data.
/// 
/// Primarily used for passing data to and from the host environment.
/// It is not reccomended to construct this directly, instead use `leak_to_shared_memory`
/// 
/// The maximum offset and size representable is 4GB each
#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct FatPointer(pub u64);

impl FatPointer {
    /// Location in memory where the data is stored
    pub fn offset(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Size of the data in bytes
    pub fn size(&self) -> u32 {
        self.0 as u32
    }

    /// Copies the data from the fat pointer into a newly allocated `Box<[u8]>`.
    /// 
    /// This is unsafe because it assumes that the data at the offset is valid. It is
    /// the caller's responsibility to ensure that this is the case.
    pub unsafe fn copy_data(&self) -> Box<[u8]> {
        let slice = from_raw_parts(
            self.offset() as *const u8,
            self.size() as usize
        );
        Box::from(slice)
    }
}

/// Creates a new allocation in memory and copies the provided data into it.
/// Returns a `FatPointer` that points to the newly allocated memory.
///
/// This allocation is not tied to any lifetime and is the caller's responsibility
/// to manage manually.
pub fn leak_to_shared_memory(value: &[u8]) -> FatPointer {
    let layout = Layout::array::<u8>(value.len()).unwrap();
    unsafe {
        let new_ptr = std::alloc::alloc(layout);
        core::ptr::copy(value.as_ptr(), new_ptr, value.len());
        FatPointer((new_ptr as u64) << 32u64 | value.len() as u64)
    }
}
