use std::io::Write;

use serde::Deserialize;

use crate::memory::leak_to_shared_memory;

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn bufferLog(offset: u32, size: u32);
    pub fn consoleLog(offset: u32, size: u32);
}

/// A writer that buffers log output and sends it to the host environment
/// to be included in the attestation.
#[derive(Default)]
pub struct LogWriter {
    buffer: Vec<u8>
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        std::io::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let ptr = leak_to_shared_memory(&self.buffer);
        unsafe { bufferLog(ptr.offset(), ptr.size()) };
        self.buffer.clear();
        std::io::Result::Ok(())
    }
}

/// Basically a print! macro for sending logs to be included in the attestation.
/// 
/// Example:
/// ```
/// log!("Formatted Log Output, {}", 32);
/// ```
/// 
/// Unlike print! this will flush the output immediately after every call.
#[macro_export]
macro_rules! log {
    ($fmt:expr $(, $arg:expr)*) => {
        let mut log_output = LogWriter::default();
        writeln!(log_output, $fmt $(, $arg)*).unwrap();
        log_output.flush().unwrap();
    };
}

/// A writer that buffers log output and sends it to the host environment
/// to be printed to the console.
#[derive(Default)]
pub struct HostWriter {
    buffer: Vec<u8>
}

impl Write for HostWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        std::io::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let ptr = leak_to_shared_memory(&self.buffer);
        unsafe { consoleLog(ptr.offset(), ptr.size()) };
        self.buffer.clear();
        std::io::Result::Ok(())
    }
}

/// Basically a print! macro for sending logs that will be printed to the console.
/// 
/// Example:
/// ```
/// host_log!("Formatted Log Output, {}", 32);
/// ```
/// 
/// Unlike print! this will flush the output immediately after every call.
#[macro_export]
macro_rules! host_log {
    ($fmt:expr $(, $arg:expr)*) => {
        let mut host_output = HostWriter::default();
        writeln!(host_output, $fmt $(, $arg)*).unwrap();
        host_output.flush().unwrap();
    };
}

/// Host functions tend to use this result type in JSON format to return data to the program.
/// In the case of an error, the value of `value` will be zeroed.
#[derive(Deserialize, Debug)]
pub struct HostResult<T> {
    pub ok: bool,
    pub error: String,
    pub value: T
}

/// Context for the entrypoint function, containing the input and secrets.
#[derive(Deserialize, Debug)]
pub struct Context<I=(), S=()> {
    pub input: I,
    pub secrets: S
}
