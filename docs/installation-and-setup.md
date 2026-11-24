---
title: Installation
weight: 3
---

## Requirements

Requires Rust 1.89 or newer. Install Rust using [rustup](https://rustup.rs/).

Behind the scenes this package uses [pdftotext](https://en.wikipedia.org/wiki/Pdftotext).
You can verify if the binary is installed on your system using this command:

```sh
which pdftotext
```

On Windows, use `where.exe pdftotext` instead. If it is installed, this returns
the path to the binary.

To install the binary you can use this command on Ubuntu or Debian:

```sh
apt-get install poppler-utils
```

On a Mac you can install the binary using brew:

```sh
brew install poppler
```

If you're on RedHat, CentOS, Rocky Linux or Fedora use this:

```sh
yum install poppler-utils
```

If you're on Windows you can download the
[Xpdf command line tools](https://www.xpdfreader.com/download.html) and store
their contents at `C:\Program Files\xpdf-tools-win`.
The executable should be `C:\Program Files\xpdf-tools-win\bin64\pdftotext.exe`.
Alternatively, install Poppler and add the directory containing `pdftotext.exe`
to PATH.

## Installation

You can install the package using Cargo:

```sh
cargo add spatie-pdf-to-text
```

Continue with [usage](usage.md).
