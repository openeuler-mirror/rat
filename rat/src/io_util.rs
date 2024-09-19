//!This file is part of rat
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
// Reference: https://github.com/coreutils/coreutils/blob/master/src/ioblksize.h

use crossbeam::channel::{self, Sender};
use std::{
    io::{self, Result, Write},
    mem,
    thread::{self, JoinHandle},
};

use nix::libc::{stat, S_IFMT, S_IFREG};

pub const IO_BUFSIZE: usize = 256 * 1024;

/// Determine if the program is running in a multithreaded environment
pub fn is_multithread() -> bool {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        > 1
}

/// Determine the optimal block size for I/O operations
pub fn io_blksize(stat: &stat) -> usize {
    let mut blksize: usize = if stat.st_blksize > 0 {
        stat.st_blksize as usize
    } else {
        IO_BUFSIZE
    };

    blksize += (IO_BUFSIZE - 1) - (IO_BUFSIZE - 1) % blksize;

    // If the file is a regular file and the block size is not a power of 2,
    // round up to the next power of 2.
    if (stat.st_mode & S_IFMT) == S_IFREG && (blksize & (blksize - 1)) != 0 {
        let leading_zeros = blksize.leading_zeros();

        if leading_zeros != 0 {
            blksize = 1usize << (usize::BITS - leading_zeros);
        }
    }

    blksize
}

/// A buffered writer that can be used in a multithreaded environment
pub struct BufferedWriterMultiThread {
    buffer: Vec<u8>,
    max_size: usize,
    sender: Sender<Vec<u8>>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl BufferedWriterMultiThread {
    /// Create a new BufferedWriter
    pub fn new() -> Self {
        let (sender, receiver) = channel::unbounded::<Vec<u8>>();

        let handle = thread::spawn(move || -> Result<()> {
            let mut writer = io::stdout();
            while let Ok(buffer) = receiver.recv() {
                if buffer.is_empty() {
                    break;
                }
                writer.write_all(&buffer)?;
            }
            Ok(())
        });

        Self {
            buffer: Vec::with_capacity(IO_BUFSIZE),
            max_size: IO_BUFSIZE,
            sender,
            handle: Some(handle),
        }
    }

    /// Write data to the buffer
    pub fn write(&mut self, mut data: &[u8]) -> Result<()> {
        while self.buffer.len() + data.len() > self.max_size {
            let process_len = self.max_size - self.buffer.len();
            self.buffer.extend_from_slice(&data[..process_len]);
            data = &data[process_len..];
            self.flush()?;
        }
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Write a single byte to the buffer
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.buffer.len() == self.buffer.capacity() {
            self.flush()?;
        }
        self.buffer.push(byte);
        Ok(())
    }

    /// Flush the buffer to the writer
    pub fn flush(&mut self) -> Result<()> {
        self.sender.send(mem::take(&mut self.buffer)).unwrap();
        self.buffer = Vec::with_capacity(IO_BUFSIZE);
        Ok(())
    }

    /// Wait for the writer to finish
    pub fn wait(&mut self) -> Result<()> {
        self.sender.send(Vec::new()).unwrap();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap()?;
        }

        Ok(())
    }
}

/// A buffered writer that can be used in a single thread environment
pub struct BufferedWriterSingleThread {
    buffer: Vec<u8>,
    max_size: usize,
}

impl BufferedWriterSingleThread {
    /// Create a new BufferedWriter
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(IO_BUFSIZE),
            max_size: IO_BUFSIZE,
        }
    }

    /// Write data to the buffer
    pub fn write(&mut self, mut data: &[u8]) -> Result<()> {
        while self.buffer.len() + data.len() > self.max_size {
            let process_len = self.max_size - self.buffer.len();
            self.buffer.extend_from_slice(&data[..process_len]);
            data = &data[process_len..];
            self.flush()?;
        }
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Write a single byte to the buffer
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.buffer.len() == self.buffer.capacity() {
            self.flush()?;
        }
        self.buffer.push(byte);
        Ok(())
    }

    /// Flush the buffer to the writer
    pub fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            io::stdout().write_all(&self.buffer)?;
            self.buffer.clear();
        }
        Ok(())
    }

    /// Single thread wait do nothing
    pub fn wait(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A buffered writer that can be used in a single or multithreaded environment
pub enum BufferedWriter {
    SingleThread(BufferedWriterSingleThread),
    MultiThread(BufferedWriterMultiThread),
}

impl BufferedWriter {
    /// Create a new BufferedWriter
    pub fn new() -> Self {
        if is_multithread() {
            BufferedWriter::MultiThread(BufferedWriterMultiThread::new())
        } else {
            BufferedWriter::SingleThread(BufferedWriterSingleThread::new())
        }
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        match self {
            BufferedWriter::SingleThread(writer) => writer.write(data),
            BufferedWriter::MultiThread(writer) => writer.write(data),
        }
    }

    /// Write a single byte to the buffer
    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        match self {
            BufferedWriter::SingleThread(writer) => writer.write_byte(byte),
            BufferedWriter::MultiThread(writer) => writer.write_byte(byte),
        }
    }

    /// Flush the buffer to the writer
    pub fn flush(&mut self) -> Result<()> {
        match self {
            BufferedWriter::SingleThread(writer) => writer.flush(),
            BufferedWriter::MultiThread(writer) => writer.flush(),
        }
    }

    /// Wait for the writer to finish
    pub fn wait(&mut self) -> Result<()> {
        match self {
            BufferedWriter::SingleThread(writer) => writer.wait(),
            BufferedWriter::MultiThread(writer) => writer.wait(),
        }
    }
}