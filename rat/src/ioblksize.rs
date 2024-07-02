//!This file is part of rat
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
// Reference: https://github.com/coreutils/coreutils/blob/master/src/ioblksize.h

use nix::libc::{stat, S_IFMT, S_IFREG};

const IO_BUFSIZE: usize = 256 * 1024;

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
