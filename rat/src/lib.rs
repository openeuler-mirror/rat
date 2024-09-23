//!This file is part of rat
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

mod io_util;
use clap::{crate_version, Arg, ArgAction, ArgMatches, Command, Error};
use io_util::{io_blksize, BufferedWriter};
use nix::fcntl::{fcntl, FcntlArg};
use nix::libc::{lseek, O_APPEND, SEEK_CUR};
use nix::sys::stat::fstat;
use nix::unistd::isatty;
use std::os::fd::RawFd;
use std::os::unix::io::AsRawFd;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

const ABOUT: &str = "rat - concatenate files and print on the standard output";

const LINE_COUNTER_BUF_LEN: usize = 20;

#[derive(PartialEq, Eq)]
/// Enum representing the mode for numbering lines
pub enum NumberMode {
    /// No line number
    None,
    /// Number all output lines
    NonBlank,
    /// Number all nonempty output lines
    AllLine,
}

/// Configuration struct for the `rat` program.
pub struct Config {
    /// line number mode
    pub number_mode: NumberMode,
    /// display $ at end of each line
    pub show_ends: bool,
    /// suppress repeated empty output lines
    pub squeeze_blank: bool,
    /// display TAB characters as ^I
    pub show_tabs: bool,
    /// use ^ and M- notation, except for LFD and TAB
    pub show_nonprinting: bool,
    /// input files
    pub files: Vec<String>,
}

/// Options for the `rat` program
pub mod options {
    ///
    pub static SHOW_ALL: &str = "show-all";
    ///
    pub static NUMBER_NONBLANK: &str = "number-nonblank";
    ///
    pub static SHOW_NONPRINTING_ENDS: &str = "e";
    ///
    pub static SHOW_ENDS: &str = "show-ends";
    ///
    pub static NUMBER: &str = "number";
    ///
    pub static SQUEEZE_BLANK: &str = "squeeze-blank";
    ///
    pub static SHOW_NONPRINTING_TABS: &str = "t";
    ///
    pub static SHOW_TABS: &str = "show-tabs";
    ///
    pub static IGNORED: &str = "u";
    ///
    pub static SHOW_NONPRINTING: &str = "show-nonprinting";
    ///
    pub static FILES: &str = "files";
}

///
impl Config {
    /// parse command line arguments
    pub fn from(matches: &ArgMatches) -> Self {
        // get number mode
        let number_mode: NumberMode = if matches.get_flag(options::NUMBER_NONBLANK) {
            NumberMode::NonBlank
        } else if matches.get_flag(options::NUMBER) {
            NumberMode::AllLine
        } else {
            NumberMode::None
        };

        let show_ends = [
            options::SHOW_ALL,
            options::SHOW_NONPRINTING_ENDS,
            options::SHOW_ENDS,
        ]
        .iter()
        .any(|&f| matches.get_flag(f));

        let squeeze_blank = matches.get_flag(options::SQUEEZE_BLANK);

        let show_tabs = [
            options::SHOW_ALL,
            options::SHOW_NONPRINTING_TABS,
            options::SHOW_TABS,
        ]
        .iter()
        .any(|&f| matches.get_flag(f));

        let show_nonprinting = [
            options::SHOW_ALL,
            options::SHOW_NONPRINTING,
            options::SHOW_NONPRINTING_ENDS,
            options::SHOW_NONPRINTING_TABS,
        ]
        .iter()
        .any(|&f| matches.get_flag(f));

        let files = match matches.get_many::<String>(options::FILES) {
            Some(f) => f.cloned().collect(),
            None => vec!["-".to_string()],
        };

        Self {
            number_mode,
            show_ends,
            squeeze_blank,
            show_tabs,
            show_nonprinting,
            files,
        }
    }

    /// Checks if the configuration allows for easy writing
    fn can_easy_write(&self) -> bool {
        !(self.show_tabs
            || self.show_nonprinting
            || self.show_ends
            || self.squeeze_blank
            || self.number_mode != NumberMode::None)
    }

    /// Returns the end-of-line string based on the configuration
    fn end_str(&self) -> &[u8] {
        if self.show_ends {
            &[b'$', b'\n']
        } else {
            &[b'\n']
        }
    }

    /// Returns the tab string based on the configuration
    fn tab_str(&self) -> &[u8] {
        if self.show_tabs {
            &[b'^', b'I']
        } else {
            &[b'\t']
        }
    }
}

/// Sets up the command-line interface for the `rat` program
pub fn rat_app() -> Command<'static> {
    Command::new("rat")
        .version(crate_version!())
        .about(ABOUT)
        .infer_long_args(true)
        // Format arguments
        .arg(
            Arg::new(options::SHOW_ALL)
                .short('A')
                .long(options::SHOW_ALL)
                .action(ArgAction::SetTrue)
                .help("equivalent to -vET"),
        )
        .arg(
            Arg::new(options::NUMBER_NONBLANK)
                .short('b')
                .long(options::NUMBER_NONBLANK)
                .action(ArgAction::SetTrue)
                .help("number nonempty output lines, overrides -n"),
        )
        .arg(
            Arg::new(options::SHOW_NONPRINTING_ENDS)
                .short('e')
                .action(ArgAction::SetTrue)
                .help("equivalent to -vE"),
        )
        .arg(
            Arg::new(options::SHOW_ENDS)
                .short('E')
                .long(options::SHOW_ENDS)
                .action(ArgAction::SetTrue)
                .help("display $ at end of each line"),
        )
        .arg(
            Arg::new(options::NUMBER)
                .short('n')
                .long(options::NUMBER)
                .action(ArgAction::SetTrue)
                .help("number all output lines"),
        )
        .arg(
            Arg::new(options::SQUEEZE_BLANK)
                .short('s')
                .long(options::SQUEEZE_BLANK)
                .action(ArgAction::SetTrue)
                .help("suppress repeated empty output lines"),
        )
        .arg(
            Arg::new(options::SHOW_NONPRINTING_TABS)
                .short('t')
                .long(options::SHOW_NONPRINTING_TABS)
                .action(ArgAction::SetTrue)
                .help("equivalent to -vT"),
        )
        .arg(
            Arg::new(options::SHOW_TABS)
                .short('T')
                .long(options::SHOW_TABS)
                .action(ArgAction::SetTrue)
                .help("display TAB characters as ^I"),
        )
        .arg(
            Arg::new(options::IGNORED)
                .short('u')
                .action(ArgAction::SetTrue)
                .help("(ignored)"),
        )
        .arg(
            Arg::new(options::SHOW_NONPRINTING)
                .short('v')
                .long(options::SHOW_NONPRINTING)
                .action(ArgAction::SetTrue)
                .help("use ^ and M- notation, except for LFD and TAB"),
        )
        .arg(
            Arg::new(options::FILES)
                .multiple(true)
                .action(ArgAction::Append),
        )
}

/// Parses command-line arguments and returns a `Config` struct
pub fn parse_cmd_args() -> Result<Config, Error> {
    let command = rat_app();
    let matches = command.get_matches();
    Ok(Config::from(&matches))
}

/// Enum representing the type of input
enum InputType {
    Stdin,
    File,
}

/// Determines the type of input based on the filename
fn get_input_type(filename: &str) -> InputType {
    if filename == "-" {
        InputType::Stdin
    } else {
        InputType::File
    }
}

struct LineNumber {
    line_buf: Vec<u8>,
    line_num_start: usize,
    line_num_end: usize,
    line_num_print: usize,
}

impl LineNumber {
    fn new(size: usize) -> Self {
        let mut line_buf = vec![b' '; size];
        let line_num_end = size - 1;
        let line_num_start = line_num_end;
        let line_num_print = line_num_end - 5;
        line_buf[line_num_end] = b'0';
        LineNumber {
            line_buf,
            line_num_start,
            line_num_end,
            line_num_print,
        }
    }

    fn next_line_num(&mut self) {
        let mut endp = self.line_num_end;
        loop {
            if self.line_buf[endp] < b'9' {
                self.line_buf[endp] += 1;
                return;
            }
            self.line_buf[endp] = b'0';
            if endp == self.line_num_start {
                break;
            }
            endp -= 1;
        }

        if self.line_num_start > 0 {
            self.line_num_start -= 1;
            self.line_buf[self.line_num_start] = b'1';
        } else {
            self.line_buf[0] = b'>';
        }

        if self.line_num_start < self.line_num_print {
            self.line_num_print -= 1;
        }
    }

    fn get_line_num(&self) -> &[u8] {
        &self.line_buf[self.line_num_print..=self.line_num_end]
    }
}

/// Struct representing the state of the output
struct OutState {
    // line: i32,
    new_line: bool,
    has_blank_line: bool,
    pre_carriage_return: bool,
    // blank_lines: i32,
    line_number: LineNumber,
}

/// Handles the input and processes it based on the configuration
fn rat_handle(
    input_state: &mut InputState,
    state: &mut OutState,
    config: &Config,
) -> Result<(), Error> {
    if config.can_easy_write() {
        easy_write(input_state)?;
    } else {
        real_write(input_state, config, state)?;
    }
    Ok(())
}

struct InputState {
    reader: Box<dyn Read + Send>,
    fd: RawFd,
    bufsize: usize,
    is_interactive: bool,
}

/// Opens a file and returns a reader
fn open_file(file: &str) -> Option<InputState> {
    let stdout = io::stdout();
    let stdout_stat = fstat(stdout.as_raw_fd());
    let stdout_flags = fcntl(stdout.as_raw_fd(), FcntlArg::F_GETFL).unwrap();

    match get_input_type(file) {
        InputType::Stdin => Some(InputState {
            reader: Box::new(io::stdin()),
            fd: io::stdin().as_raw_fd(),
            bufsize: 10240,
            is_interactive: isatty(io::stdin().as_raw_fd()).unwrap_or(false),
        }),
        InputType::File => {
            if !Path::new(file).exists() {
                eprintln!("rat: {}: No such file or directory", file);
                return None;
            }
            if Path::new(file).is_dir() {
                eprintln!("rat: {}: Is a directory", file);
                return None;
            }
            match File::open(file) {
                Ok(f) => match fstat(f.as_raw_fd()) {
                    Ok(in_stat) => {
                        if in_stat.st_dev == stdout_stat.unwrap().st_dev
                            && in_stat.st_ino == stdout_stat.unwrap().st_ino
                        {
                            let mut exhausting =
                                stdout_flags >= 0 && (stdout_flags & O_APPEND) != 0;
                            if !exhausting {
                                let in_pos = unsafe { lseek(f.as_raw_fd(), 0, SEEK_CUR) };
                                let out_pos = unsafe { lseek(stdout.as_raw_fd(), 0, SEEK_CUR) };
                                if in_pos >= 0 {
                                    exhausting = in_pos < out_pos
                                }
                            }

                            if exhausting {
                                eprintln!("rat: {}: input file is output file", file);
                                return None;
                            }
                        }
                        let fd = f.as_raw_fd();
                        Some(InputState {
                            reader: Box::new(f),
                            fd,
                            bufsize: io_blksize(&in_stat),
                            is_interactive: false,
                        })
                    }
                    _ => None,
                },
                Err(_) => {
                    eprintln!("rat: {}: error opening file", file);
                    None
                }
            }
        }
    }
}

/// Processes the input files based on the configuration
pub fn rat_process(config: &Config) -> i32 {
    let mut exit_status = 0;
    let mut out_state = OutState {
        new_line: true,
        has_blank_line: false,
        pre_carriage_return: false,
        line_number: LineNumber::new(LINE_COUNTER_BUF_LEN),
    };

    for file in &config.files {
        if let Some(mut in_stat) = open_file(file) {
            rat_handle(&mut in_stat, &mut out_state, config).unwrap_or_else(|_| {
                exit_status = 1;
            });
        } else {
            exit_status = 1;
        }
    }
    if out_state.pre_carriage_return {
        print!("\r");
    }
    exit_status
}

/// Writes the input directly to the output
fn easy_write(input_state: &mut InputState) -> Result<(), Error> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        if !input_state.is_interactive
            && !io_util::is_multithread()
            && io_util::splice_copy(input_state.fd, io::stdout().as_raw_fd())?
        {
            return Ok(());
        }
    }

    let mut buffer: Vec<u8> = vec![0; input_state.bufsize];
    let is_interactive = input_state.is_interactive;
    let mut writer = BufferedWriter::new();

    loop {
        let bytes_read = input_state.reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write(&buffer[..bytes_read])?;
        if is_interactive {
            writer.flush()?;
        }
    }
    writer.flush()?;
    writer.wait()?;
    Ok(())
}

/// Processes the input and writes it to the output with formatting
fn real_write(
    input_state: &mut InputState,
    config: &Config,
    state: &mut OutState,
) -> Result<(), Error> {
    let bufsize = input_state.bufsize;
    let reader = &mut input_state.reader;
    let is_interactive = input_state.is_interactive;

    let mut buffer: Vec<u8> = vec![0; bufsize];
    let mut writer = BufferedWriter::new();

    loop {
        let byte_read = reader.read(&mut buffer)?;

        if byte_read == 0 {
            break;
        }
        let mut offset = 0;

        while offset < byte_read {
            if buffer[offset] == b'\n' {
                if !(state.new_line && config.squeeze_blank && state.has_blank_line) {
                    if state.new_line && config.number_mode == NumberMode::AllLine {
                        state.line_number.next_line_num();
                        writer.write(state.line_number.get_line_num())?;
                        writer.write_byte(b'\t')?;
                    }

                    write_end(&mut writer, config, state, is_interactive)?;
                    state.has_blank_line = state.new_line;
                }
                offset += 1;
                state.new_line = true;
                continue;
            }
            state.has_blank_line = false;
            if state.pre_carriage_return {
                writer.write_byte(b'\r')?;
                state.pre_carriage_return = false;
                state.new_line = false;
            }
            if state.new_line && config.number_mode != NumberMode::None {
                // print line number
                state.line_number.next_line_num();
                writer.write(state.line_number.get_line_num())?;
                writer.write_byte(b'\t')?;
            }

            // write line
            let len = {
                let in_buf: &[u8] = &buffer[offset..byte_read];
                match (config.show_nonprinting, config.show_tabs) {
                    (true, _) => write_line_nonprinting(&mut writer, in_buf, config),
                    (false, true) => write_line_show_tab(&mut writer, in_buf, config),
                    _ => match in_buf.iter().position(|c| *c == b'\n' || *c == b'\r') {
                        Some(p) => {
                            writer.write(&in_buf[..p])?;
                            Ok(p)
                        }
                        None => {
                            writer.write(in_buf)?;
                            Ok(in_buf.len())
                        }
                    },
                }
            }?;

            if len > 0 {
                state.new_line = false;
            }

            if offset + len == byte_read {
                break;
            }

            match buffer[offset + len] {
                b'\n' => {
                    write_end(&mut writer, config, state, is_interactive)?;
                    state.has_blank_line = state.new_line;
                    state.new_line = true;
                }
                b'\r' => {
                    state.pre_carriage_return = true;
                    state.new_line = false;
                }
                _ => {}
            }

            offset += len + 1;
        }
    }
    writer.flush()?;
    writer.wait()?;
    Ok(())
}

/// Writes the end-of-line characters
fn write_end(
    writer: &mut BufferedWriter,
    config: &Config,
    state: &mut OutState,
    is_interactive: bool,
) -> Result<(), Error> {
    if state.pre_carriage_return {
        if config.show_ends {
            writer.write(b"^M")?;
        } else {
            writer.write(b"\r")?;
        }
        state.pre_carriage_return = false;
    }

    writer.write(config.end_str())?;
    if is_interactive {
        writer.flush()?;
    }
    Ok(())
}

/// Writes a line with non-printing characters
fn write_line_nonprinting(
    writer: &mut BufferedWriter,
    mut in_buf: &[u8],
    config: &Config,
) -> Result<usize, Error> {
    let mut pos = 0;
    loop {
        match in_buf.iter().position(|c| *c < 32 || *c > 126) {
            Some(p) => {
                writer.write(&in_buf[..p])?;
                let byte = in_buf[p];
                match byte {
                    b'\n' => return Ok(pos + p),
                    b'\t' => writer.write(config.tab_str())?,
                    32..=126 => writer.write(&[byte])?,
                    127 => writer.write(&[b'^', b'?'])?,
                    128..=159 => writer.write(&[b'M', b'-', b'^', byte - 64])?,
                    160..=254 => writer.write(&[b'M', b'-', byte - 128])?,
                    255.. => writer.write(&[b'M', b'-', b'^', b'?'])?,
                    _ => writer.write(&[b'^', byte + 64])?,
                };
                pos += p + 1;
                in_buf = &in_buf[p + 1..];
            }
            None => {
                writer.write(in_buf)?;
                return Ok(pos + in_buf.len());
            }
        }
    }
}

/// Writes a line with tab characters shown
fn write_line_show_tab(
    writer: &mut BufferedWriter,
    mut in_buf: &[u8],
    config: &Config,
) -> Result<usize, Error> {
    let mut pos = 0;
    loop {
        match in_buf
            .iter()
            .position(|c| *c == b'\t' || *c == b'\n' || *c == b'\r')
        {
            Some(p) => {
                if in_buf[p] == b'\t' {
                    writer.write(&in_buf[..p])?;
                    writer.write(config.tab_str())?;
                    in_buf = &in_buf[p + 1..];
                    pos += p + 1;
                } else {
                    writer.write(&in_buf[..p])?;
                    return Ok(pos + p);
                }
            }
            None => {
                writer.write(in_buf)?;
                return Ok(pos + in_buf.len());
            }
        }
    }
}
