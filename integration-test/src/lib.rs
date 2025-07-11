use basm_rust_sdk::{
    host_log,
    io::{output_data, HostWriter, LogWriter},
    log,
    memory::FatPointer
};
use std::io::Write;

#[no_mangle]
pub extern "C" fn hello_world(input_ptr: FatPointer, secret_ptr: FatPointer) -> FatPointer {
    log!(
        "Formatted Log Output, {}",
        32
    );
    log!(
        "Logging input {}",
        str::from_utf8(&input_ptr.copy_data()).unwrap()
    );

    host_log!(
        "Printing secrets for debug: {}",
        str::from_utf8(&secret_ptr.copy_data()).unwrap()
    );

    return output_data("Output of the fn!".as_bytes());
}
