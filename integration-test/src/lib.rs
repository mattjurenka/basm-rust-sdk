use basm_rust_sdk::leak_to_shared_memory;



#[no_mangle]
pub extern "C" fn hello_world(input_ptr: u64, secret_ptr: u64) -> u64 {
    return 0;
    //return leak_to_shared_memory("Hello World.".as_bytes());
}

#[no_mangle]
pub extern "C" fn main() {}