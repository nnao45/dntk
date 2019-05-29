# dntk
[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v2.1.1](https://img.shields.io/badge/package-v2.1.1-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v2.1.1)
[![crates](https://img.shields.io/badge/crates.io-v2.1.1-319e8c.svg)](https://crates.io/crates/dntk)
[![docker](https://img.shields.io/badge/docker-v2.1.1-blue.svg)](https://hub.docker.com/r/nnao45/dntk/tags)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![platform](https://img.shields.io/badge/platform-%20osx%20|%20linux-orange.svg)]()

dntk is command line's ***Interactive*** calculator, [GNU bc](https://www.gnu.org/software/bc/) wrapper.  
[![asciicast](https://asciinema.org/a/248298.svg)](https://asciinema.org/a/248298)
  
✔︎ dntk means calculator in a japanese.  
✔︎ dntk is gnu bc wrapper. so, syntax is equal to gnu bc. [learn syntax more](https://www.gnu.org/software/bc/manual/html_mono/bc.html)  
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

## ***Current dntk's version: v2.1.1***
Download Page: https://github.com/nnao45/dntk/releases/latest

## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.1.1/dntk-v2.1.1-x86_64-unknown-linux-gnu.zip
$ unzip dntk-v2.1.1-x86_64-unknown-linux-gnu.zip
```

if can't move, try...
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.1.1/dntk-v2.1.1-x86_64-unknown-linux-musl.zip
$ unzip dntk-v2.1.1-x86_64-unknown-linux-musl.zip
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
dntk 2.1.2
nnao45 <n4sekai5y@gmail.com>
Command line's Interactive calculator, GNU bc wrapper.

USAGE:
    dntk [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      No print information message
    -v, --version    Prints version information
    -w, --white      Set White color in a output

OPTIONS:
        --bc-path <PATH>    Use a specific bc command path [default: bc]
    -s, --scale <NUMBER>    Number of truncated after the decimal point
```

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
