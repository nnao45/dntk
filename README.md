[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v2.0.1](https://img.shields.io/badge/package-v2.0.1-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v2.0.1)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)

# dntk
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

## clean buffer
if you want to clean buffer, very easy, type `r`

![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-demo05.gif)

## Pipe use...
```bash
$ echo "123 * 2" | dntk | xargs echo
246
```
behave, like bc ☺️

***Have a nice go hacking days***:sparkles::wink:
## Writer & License
dntk was writed by nnao45 (WORK:Infrastructure Engineer, Twitter:@nnao45, MAIL:n4sekai5y@gmail.com).  
This software is released under the MIT License, see LICENSE.
