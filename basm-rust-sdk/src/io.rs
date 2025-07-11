use std::{default, io::Write, sync::{Arc, Mutex}};

use crate::memory::{leak_to_shared_memory, FatPointer};

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn bufferLog(offset: u32, size: u32);
    pub fn consoleLog(offset: u32, size: u32);
}

#[derive(Default)]
pub struct LogWriter {
    buffer: Vec<u8>
}

// assumes that data will be well within what u32s are capable of
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

#[macro_export]
macro_rules! log {
    ($fmt:expr $(, $arg:expr)*) => {
        let mut log_output = LogWriter::default();
        writeln!(log_output, $fmt $(, $arg)*).unwrap();
        log_output.flush().unwrap();
    };
}

#[derive(Default)]
pub struct HostWriter {
    buffer: Vec<u8>
}

// assumes that data will be well within what u32s are capable of
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

#[macro_export]
macro_rules! host_log {
    ($fmt:expr $(, $arg:expr)*) => {
        let mut host_output = HostWriter::default();
        writeln!(host_output, $fmt $(, $arg)*).unwrap();
        host_output.flush().unwrap();
    };
}

pub fn output_data(value: &[u8]) -> FatPointer {
    leak_to_shared_memory(value)
}