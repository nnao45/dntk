# dntk
[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v2.2.1](https://img.shields.io/badge/package-v2.2.1-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v2.2.1)
[![crates](https://img.shields.io/badge/crates.io-v2.2.1-319e8c.svg)](https://crates.io/crates/dntk)
[![docker](https://img.shields.io/badge/docker-v2.2.1-blue.svg)](https://hub.docker.com/r/nnao45/dntk/tags?name=v2.2.1)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![platform](https://img.shields.io/badge/platform-%20osx%20|%20linux%20|%20freebsd%20|%20windows-orange.svg)](https://github.com/nnao45/dntk/releases)

dntk is command line's multi-platform ***Interactive*** calculator with bc-compatible syntax and **high-precision arithmetic**.
![gjf](https://github.com/nnao45/naoGifRepo/blob/master/dntk_demo.gif)

✔︎ dntk means calculator in a japanese.
✔︎ dntk is bc-compatible calculator with **28-digit precision** (no external bc required!)
✔︎ dntk syntax is compatible with [GNU bc](https://www.gnu.org/software/bc/). [learn syntax more](https://www.gnu.org/software/bc/manual/html_mono/bc.html)
✔︎ dntk is a NATIVE [The Rust Programming Language](https://rust-lang.org) application.
✔︎ dntk can move cursor, can delete char, can refresh buffer.
✔︎ dntk provides **accurate decimal arithmetic** without floating-point errors.  
✔︎ dntk write color means,  
<table>
    <tr>
        <td>color</td>
        <td>means</td>
    </tr>
    <tr>
        <td>cyan</td>
        <td>can caluclate & can output</td>
    </tr>
    <tr>
        <td>megenta</td>
        <td>can't caluclate, can't output</td>
    </tr>
    <tr>
        <td>yellow</td>
        <td>danger input char, output warning</td>
    </tr>
    <tr>
        <td>green</td>
        <td>clean buffer message</td>
    </tr>
</table>

## ***Current dntk's version:v2.2.1***
Download Page: https://github.com/nnao45/dntk/releases/latest

## ✨ Key Features

### 🎯 High-Precision Arithmetic
- **28 decimal digits** of precision (exceeds bc's default 20 digits)
- **No floating-point errors**: `1 + 0.7 = 1.7` (not 1.69999...)
- Accurate division: `1/3 = .33333333333333333333`

### ⚡ Fast & Lightweight
- **No external dependencies** (bc command not required!)
- Pure Rust implementation for maximum performance
- Optimized expression evaluation with `fasteval` + `rust_decimal`

### 🌍 True Cross-Platform
- Works out of the box on **Windows, Linux, macOS, and FreeBSD**
- No need to install bc.exe on Windows anymore!
- Single binary, easy deployment

### 🔧 bc-Compatible
- Supports standard bc syntax and functions
- `s()`, `c()`, `a()`, `l()`, `e()`, `sqrt()` functions
- Interactive REPL with cursor movement and editing

## Platform
dntk support multi-platform 😊 mac, linux, freebsd, and **windows**!!!
- i686-osx
- x86_64-osx
- i686-linux
- x86_64-linux
- i686-windows
- x86_64-windows
- i686-freebsd
- x86_64-freebsd
## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.2.1/dntk-v2.2.1-x86_64-unknown-linux-musl.zip
$ unzip dntk-v2.2.1-x86_64-unknown-linux-musl.zip
```

### Windows
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.2.1/dntk-v2.2.1-x86_64-pc-windows-msvc.zip
$ unzip dntk-v2.2.1-x86_64-pc-windows-msvc.zip
```

### FreeBSD
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.2.1/dntk-v2.2.1-x86_64-unknown-freebsd.zip
$ unzip dntk-v2.2.1-x86_64-unknown-freebsd.zip
```

### Cargo
```bash
$ cargo install dntk
```

### zplug
```bash
$ zplug 'nnao45/dntk', as:command, from:gh-r
```

### Docker
Can use dntk docker image,  
Look!! Very light weight!!🚀

```bash
$ docker images nnao45/dntk
REPOSITORY          TAG                 IMAGE ID            CREATED             SIZE
nnao45/dntk         latest              3a37b5d989b5        2 hours ago         10.5MB
```

And run, 

```bash
$ docker run -it --rm nnao45/dntk:latest
```

### And...
```bash
$ echo 'alias bc=dntk' >> ~/.bashrc
$ echo 'alias bc=dntk' >> ~/.zshrc
```
All OK!! 😎

## Options
```
❯❯❯ dntk -h
Command line's multi-platform interactive calculator with high-precision arithmetic.

USAGE:
    dntk [FLAGS] [OPTIONS]

FLAGS:
    -h, --help           Prints help information
        --once           Run at only once
    -q, --quiet          No print information message
        --show-limits    Print the local limits
    -V, --version        Prints version information
    -w, --white          Set White color in a output

OPTIONS:
    -i, --inject <inject>      Pre-run inject statement to the dntk [default: ]
    -s, --scale <scale>        Number of decimal places (max 28) [default: 20]
```

**Note**: `--bc-path` option has been removed as dntk no longer requires external bc command!

## Pipe Support
```bash
$ echo "123 * 2" | dntk
246
```
behave, like bc ☺️

## Paste Support
```bash
$ echo '( 1 + 2 + 3 + 4 + 51 ) / sqrt( 123 / 3 )' | pbcopy
$ pbpaste | dntk
9.52659947520496999698
```

## Windows Support
**No additional setup required!** 🎉

dntk works out of the box on Windows without installing bc.exe. Just download and run!

![gjf](https://github.com/nnao45/naoGifRepo/blob/master/dntk_win_demo.gif)

### Previous versions (v2.2.1 and earlier)
Older versions required bc.exe installation. If you're using an older version:
```bash
$ choco install gnuwin
```
**Recommendation**: Upgrade to the latest version for better Windows support!

## Keybind

### Basic Key

<table>
    <tr>
        <td>key</td>
        <td>feature</td>
    </tr>
    <tr>
        <td>[, ←</td>
        <td>cursor move to left</td>
    </tr>
    <tr>
        <td>], →</td>
        <td>cursor move to right</td>
    </tr>
    <tr>
        <td>0~9</td>
        <td>Sendkey this number</td>
    </tr>
    <tr>
        <td>Ctrl+C, Enter</td>
        <td>Finish dntk app</td>
    </tr>
    <tr>
        <td>Delete, Backspace</td>
        <td>Delete current char</td>
    </tr>
    <tr>
        <td>@</td>
        <td>Clean buffer</td>
    </tr>

</table>

### Basic Operation

<table>
    <tr>
        <td>key</td>
        <td>feature</td>
    </tr>
    <tr>
        <td>+</td>
        <td>plus</td>
    </tr>
    <tr>
        <td>-</td>
        <td>minus</td>
    </tr>
    <tr>
        <td>*</td>
        <td>multiplication</td>
    </tr>
    <tr>
        <td>/</td>
        <td>division</td>
    </tr>
    <tr>
        <td>^</td>
        <td>exponentiation</td>
    </tr>
    <tr>
        <td>%</td>
        <td>remainder</td>
    </tr>
</table>

### Operation for Logical

<table>
    <tr>
        <td>key</td>
        <td>feature</td>
    </tr>
    <tr>
        <td>!</td>
        <td>boolean, relational</td>
    </tr>
    <tr>
        <td>|</td>
        <td>boolean</td>
    </tr>
    <tr>
        <td>&</td>
        <td>boolean</td>
    </tr>
    <tr>
        <td>></td>
        <td>relational</td>
    </tr>
    <tr>
        <td><</td>
        <td>relational</td>
    </tr>
    <tr>
        <td>=</td>
        <td>relational</td>
    </tr>
</table>

### Using Function

you can use under function.

<table>
    <tr>
        <td>function</td>
        <td>key</td>
        <td>feature</td>
    </tr>
    <tr>
        <td>(x)</td>
        <td>()</td>
        <td>Simple round bracket</td>
    </tr>
    <tr>
        <td>sin(x)</td>
        <td>s()</td>
        <td>Sin of trigonometric function</td>
    </tr>
    <tr>
        <td>cos(x)</td>
        <td>c()</td>
        <td>Cosin of trigonometric function</td>
    </tr>
    <tr>
        <td>atan(x)</td>
        <td>a()</td>
        <td>Tangent of inverse trigonometric function</td>
    </tr>
    <tr>
        <td>log(x)</td>
        <td>l()</td>
        <td>Logarithm function</td>
    </tr>
    <tr>
        <td>exp(x)</td>
        <td>e()</td>
        <td>Exponential function</td>
    </tr>
    <tr>
        <td>sqrt(x)</td>
        <td>sqrt()</td>
        <td>Return square root of the expression function</td>
    </tr>
    <tr>
        <td>j(n,x)</td>
        <td>j()</td>
        <td>The n-order Bessel function</td>
    </tr>
</table>

more detail 👉 https://www.gnu.org/software/bc/manual/html_mono/bc.html

## 🔬 Technical Details

### Architecture
dntk uses a hybrid approach for optimal performance and precision:

1. **Expression Parsing**: `fasteval` - Fast and lightweight expression parser
2. **High-Precision Arithmetic**: `rust_decimal` - 28-digit decimal precision
3. **Result Formatting**: bc-compatible output format

### Precision Comparison

| Calculator | Precision | Example: 1+0.7 | Example: 1/3 (20 digits) |
|-----------|-----------|----------------|--------------------------|
| bc | 20 digits (default) | 1.7 | .33333333333333333333 |
| dntk (old) | ~15 digits (f64) | 1.69999... ❌ | .33333333333333331483 ❌ |
| **dntk (new)** | **28 digits** | **1.7 ✅** | **.33333333333333333333 ✅** |

### Dependencies
- `fasteval` - Expression evaluation
- `rust_decimal` - High-precision decimal arithmetic (up to 28 digits)
- Pure Rust implementation (no C library dependencies)

### Why No bc Command Required?
Previous versions wrapped the external `bc` command. The new version:
- Implements bc-compatible arithmetic in pure Rust
- Eliminates subprocess overhead
- Works on all platforms without external dependencies
- Provides better precision (28 vs 20 digits)

# Development Guide

## Compile

### Binary

```bash
$ make
```

### Docker

```bash
$ make docker-build
```

# Contribute

Always Welcome!! 😄

***Have a nice rust hacking days***:sparkles::wink:
## Writer & License
dntk was writed by nnao45 (WORK:Infrastructure Engineer, Twitter:@nnao45, MAIL:n4sekai5y@gmail.com).  
This software is released under the MIT License, see LICENSE.
