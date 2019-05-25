# dntk
[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v2.0.1](https://img.shields.io/badge/package-v2.0.1-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v2.0.1)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![platform](https://img.shields.io/badge/platform-%20osx%20|%20linux-blue.svg)]()

dntk is command line's ***Interactive*** calculator, [GNU bc](https://www.gnu.org/software/bc/) wrapper.  
[![asciicast](https://asciinema.org/a/248298.svg)](https://asciinema.org/a/248298)
  
✔︎ dntk means calculator in a japanese.  
✔︎ dntk is gnu bc wrapper. so, syntax is equal to gnu bc. [learn syntax more](https://www.gnu.org/software/bc/manual/html_mono/bc.html)  
✔︎ dntk is a NATIVE [The Rust Programming Language](https://rust-lang.org) application.  
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
</table>

## ***Current dntk's version: v2.0.1***
Download Page: https://github.com/nnao45/dntk/releases/latest

## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.0.1/dntk-linux-amd64-v2.0.1.tar.gz
$ tar xvfz dntk-linux-amd64-v2.0.1.tar.gz
```

### zplug
```bash
$ zplug 'nnao45/dntk', as:command, from:gh-r
```

### And...
```bash
$ echo 'alias bc=dntk' >> ~/.bashrc
$ echo 'alias bc=dntk' >> ~/.zshrc
```
All OK!! 😎

## Movable Cursor

<table>
    <tr>
        <td>key</td>
        <td>feature</td>
    </tr>
    <tr>
        <td>[</td>
        <td>cursor move to left</td>
    </tr>
    <tr>
        <td>]</td>
        <td>cursor move to right</td>
    </tr>
</table>

## Using Function

you can use under function.

<table>
    <tr>
        <td>function</td>
        <td>command</td>
        <td>detail</td>
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
        <td>j(n,x)</td>
        <td>j()</td>
        <td>The n-order Bessel function</td>
    </tr>
</table>

more detail 👉 https://www.gnu.org/software/bc/manual/html_mono/bc.html

## clean buffer
if you want to clean buffer, very easy, type `r`

[![asciicast](https://asciinema.org/a/248301.svg)](https://asciinema.org/a/248301)

## Pipe Use...
```bash
$ echo "123 * 2" | dntk | xargs echo
246
```
behave, like bc ☺️

***Have a nice rust hacking days***:sparkles::wink:
## Writer & License
dntk was writed by nnao45 (WORK:Infrastructure Engineer, Twitter:@nnao45, MAIL:n4sekai5y@gmail.com).  
This software is released under the MIT License, see LICENSE.
