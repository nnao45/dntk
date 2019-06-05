# dntk
[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![vvv2.1.12](https://img.shields.io/badge/package-vvv2.1.12-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/vvv2.1.12)
[![crates](https://img.shields.io/badge/crates.io-vvv2.1.12-319e8c.svg)](https://crates.io/crates/dntk)
[![docker](https://img.shields.io/badge/docker-vvv2.1.12-blue.svg)](https://hub.docker.com/r/nnao45/dntk/tags)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![platform](https://img.shields.io/badge/platform-%20osx%20|%20linux%20|%20windows-orange.svg)](https://github.com/nnao45/dntk/releases)

dntk is command line's multi-platform ***Interactive*** calculator, [GNU bc](https://www.gnu.org/software/bc/) wrapper.  
![gjf](https://github.com/nnao45/naoGifRepo/blob/master/dntk_demo.gif)
  
✔︎ dntk means calculator in a japanese.  
✔︎ dntk is gnu bc wrapper. so, syntax is equal to gnu bc. [learn syntax more](https://www.gnu.org/software/bc/manual/html_mono/bc.html)  
✔︎ dntk support multi-platform 😊 mac, alpine, linux, and **windows**!!!  
✔︎ dntk is a NATIVE [The Rust Programming Language](https://rust-lang.org) application.  
✔︎ dntk can move cursor, can delete char, can refresh buffer.  
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

## ***Current dntk's version: vvv2.1.12***
Download Page: https://github.com/nnao45/dntk/releases/latest

## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/vvv2.1.12/dntk-vvv2.1.12-x86_64-unknown-linux-gnu.zip
$ unzip dntk-vvv2.1.12-x86_64-unknown-linux-gnu.zip
```

if can't move, try...
```bash
$ wget https://github.com/nnao45/dntk/releases/download/vvv2.1.12/dntk-vvv2.1.12-x86_64-unknown-linux-musl.zip
$ unzip dntk-vvv2.1.12-x86_64-unknown-linux-musl.zip
```

### Windows
```bash
$ wget https://github.com/nnao45/dntk/releases/download/vvv2.1.12/dntk-vvv2.1.12-x86_64-pc-windows-msvc.zip
$ unzip dntk-vvv2.1.12-x86_64-pc-windows-msvc.zip
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
Command line's multi-platform interactive calculator, GNU bc wrapper.

USAGE:
    dntk [FLAGS] [OPTIONS]

FLAGS:
    -h, --help           Prints help information
        --once           Run at only once
    -q, --quiet          No print information message
        --show-limits    Print the local limits enforced by the local version of bc, and quit
    -V, --version        Prints version information
    -w, --white          Set White color in a output

OPTIONS:
    -b, --bc-path <bc_path>    Use a specific bc command path [default: bc]
    -i, --inject <inject>      Pre-run inject statement to the dntk [default: ]
    -s, --scale <scale>        Number of truncated after the decimal point [default: 20]
```

## Windows Support
You may install bc.exe and set PATH.
```bash
$ choco install gnuwin
$ # or
$ wget wget https://embedeo.org/ws/command_line/bc_dc_calculator_windows/bc-1.07.1-win32-embedeo-02.zip
$ unzip bc-1.07.1-win32-embedeo-02.zip
```

![gjf](https://github.com/nnao45/naoGifRepo/blob/master/dntk_win_demo.gif)

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

## Pipe Use...
```bash
$ echo "123 * 2" | dntk
246
```
behave, like bc ☺️

***Have a nice rust hacking days***:sparkles::wink:
## Writer & License
dntk was writed by nnao45 (WORK:Infrastructure Engineer, Twitter:@nnao45, MAIL:n4sekai5y@gmail.com).  
This software is released under the MIT License, see LICENSE.
