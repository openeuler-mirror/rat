Name:           rat
Version:        0.1.0
Release:        1%{?dist}
Summary:        This project is refactoring the cat tool with rust.

License:        MulanPSL2
URL:            https://gitee.com/openeuler/rat
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo

%description
The `rat` project reimplements the classic `cat` tool in Rust, 
offering better memory management and faster execution. 
Ideal for efficient file operations in Unix-like systems.

%global debug_package %{nil}

%prep
%autosetup

%build
cargo build --release --all

%install
install -D -m 0755 target/release/rat %{buildroot}/%{_bindir}/rat

%files
%{_bindir}/rat
