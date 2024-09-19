# rat - 基于 Rust 的 `cat` 替代工具

`rat` 是一个用 Rust 编写的命令行工具，设计为 `cat` 命令的替代版本。它提供了与 `cat` 相同的功能，同时还包括多线程优化以及基于 Linux 特定系统调用的高效文件处理。

## 特性

- **基本文件操作**：连接文件并输出到标准输出。
- **选项**：支持与 `cat` 相同的选项，如行号显示、行尾标记、压缩空白行等。
- **多线程执行**：针对多核系统进行了优化，处理大文件时具有更高的性能。
- **高效 I/O**：使用 Linux 的 `splice` 系统调用来提高文件复制性能，减少用户空间与内核空间之间的数据移动。

### 安装

项目提供了两种安装方式：

#### 1. 手动编译安装

首先确保系统已安装 Rust 编译环境，然后执行以下命令：

```bash
git clone https://gitee.com/openeuler/rat
cd rat
cargo build --all --release
```

完成编译后，程序会生成在 `target/release/` 目录中，您可以将生成的二进制文件拷贝到系统路径，或使用如下命令安装：

```bash
sudo cp target/release/rat /usr/local/bin/
```

#### 2. 使用 RPM 包安装

本项目还提供了 RPM 包的构建和安装方法：

1. 执行以下命令构建 RPM 包：

```bash
make rpm
```

2. 生成的 RPM 包会保存在 `~/rpmbuild/RPMS/` 目录中，接着使用以下命令安装：

```bash
make rpm-install
```

3. 若需要卸载该程序，可以运行以下命令：

```bash
make rpm-uninstall
```

## 使用方法

`rat` 命令的使用方式与 `cat` 相同，支持以下选项：

```bash
rat [OPTION]... [FILE]...
```

- `-A, --show-all`：显示所有字符，相当于 `-vET`。
- `-b, --number-nonblank`：为非空行添加行号。
- `-e`：相当于 `-vE`，显示不可打印字符，并在行尾添加 `$`。
- `-E, --show-ends`：在每行行尾显示 `$`。
- `-n, --number`：为所有输出行添加行号。
- `-s, --squeeze-blank`：压缩连续的空白行为一行。
- `-t`：相当于 `-vT`，显示不可打印字符，并将制表符显示为 `^I`。
- `-T, --show-tabs`：将制表符显示为 `^I`。
- `-v, --show-nonprinting`：使用 `^` 和 `M-` 记号显示不可打印字符。
- `--help`：显示帮助信息并退出。
- `--version`：显示版本信息并退出。

### 示例

1. 显示文件内容：
   ```bash
   rat file.txt
   ```

2. 连接多个文件并显示：
   ```bash
   rat file1.txt file2.txt
   ```

3. 显示带行号的文件内容：
   ```bash
   rat -n file.txt
   ```

4. 显示所有字符，包括制表符和行尾标记：
   ```bash
   rat -A file.txt
   ```

## 许可证

本项目基于 MulanPSL2 许可证，详见 [LICENSE](LICENSE) 文件。
