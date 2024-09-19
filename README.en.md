# rat - A Rust-Based Alternative to `cat`

`rat` is a command-line tool written in Rust, designed as a replacement for the `cat` command. It offers the same functionality as `cat`, with additional multi-threading optimizations and efficient file handling based on Linux-specific system calls.

## Features

- **Basic File Operations**: Concatenate files and output to standard output.
- **Options**: Supports the same options as `cat`, including line numbering, end-of-line markers, and blank line compression.
- **Multi-threaded Execution**: Optimized for multi-core systems, providing improved performance when handling large files.
- **Efficient I/O**: Utilizes the Linux `splice` system call to enhance file copy performance, reducing data movement between user space and kernel space.

### Installation

There are two ways to install the project:

#### 1. Manual Compilation

Ensure you have the Rust build environment installed, then execute the following commands:

```bash
git clone https://gitee.com/openeuler/rat
cd rat
cargo build --all --release
```

After compilation, the program will be generated in the `target/release/` directory. You can copy the binary to your system path or use the following command for installation:

```bash
sudo cp target/release/rat /usr/local/bin/
```

#### 2. Using RPM Package

The project also provides a method for building and installing RPM packages:

1. Build the RPM package with the following command:

```bash
make rpm
```

2. The generated RPM package will be located in the `~/rpmbuild/RPMS/` directory. Install it using:

```bash
make rpm-install
```

3. To uninstall the program, use:

```bash
make rpm-uninstall
```

## Usage

The `rat` command is used in the same way as `cat` and supports the following options:

```bash
rat [OPTION]... [FILE]...
```

- `-A, --show-all`: Display all characters, equivalent to `-vET`.
- `-b, --number-nonblank`: Number non-blank lines.
- `-e`: Equivalent to `-vE`, show non-printing characters and add `$` at end of lines.
- `-E, --show-ends`: Display `$` at the end of each line.
- `-n, --number`: Number all output lines.
- `-s, --squeeze-blank`: Compress consecutive blank lines into one.
- `-t`: Equivalent to `-vT`, display non-printing characters and show TAB as `^I`.
- `-T, --show-tabs`: Display TAB characters as `^I`.
- `-v, --show-nonprinting`: Show non-printing characters using `^` and `M-` symbols.
- `--help`: Show help information and exit.
- `--version`: Show version information and exit.

### Examples

1. Display file content:
   ```bash
   rat file.txt
   ```

2. Concatenate multiple files and display:
   ```bash
   rat file1.txt file2.txt
   ```

3. Display file content with line numbers:
   ```bash
   rat -n file.txt
   ```

4. Display all characters, including tabs and end-of-line markers:
   ```bash
   rat -A file.txt
   ```

## License

This project is licensed under the MulanPSL2 License. See the [LICENSE](LICENSE) file for details.
