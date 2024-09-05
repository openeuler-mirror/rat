//!This file is part of rat
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
// Reference: https://github.com/coreutils/coreutils/blob/master/src/ioblksize.h

use std::io::{self, Write};

use nix::libc::{stat, S_IFMT, S_IFREG};

pub const IO_BUFSIZE: usize = 256 * 1024;

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

pub struct BufferedWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> BufferedWriter<W> {
    /// Create a new BufferedWriter
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(IO_BUFSIZE),
        }
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if self.buffer.len() + data.len() > self.buffer.capacity() {
            self.flush()?;
        }
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Write a single byte to the buffer
    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.buffer.len() == self.buffer.capacity() {
            self.flush()?;
        }
        self.buffer.push(byte);
        Ok(())
    }

    /// Flush the buffer to the writer
    pub fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        Ok(())
    }

    /// Write data immediately to the writer, bypassing the buffer
    pub fn write_immediately(&mut self, data: &[u8]) -> io::Result<()> {
        self.flush()?;
        self.writer.write_all(data)
    }
}

impl<W: Write> Drop for BufferedWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
