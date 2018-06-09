[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v1.0.5](https://img.shields.io/badge/package-v1.0.5-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v1.0.5)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![Go Report Card](https://goreportcard.com/badge/github.com/nnao45/dntk)](https://goreportcard.com/report/github.com/nnao45/dntk)

# dntk
dntk is command line's **Interactive** calculator, [GNU bc](https://www.gnu.org/software/bc/) wrapper.  
![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-v105.gif)

## ***Current dntk's version: v1.0.5***
Download Page: https://github.com/nnao45/dntk/releases/latest

## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v1.0.5/dntk-linux-amd64-v1.0.5.tar.gz
$ tar xvfz dntk-linux-amd64-v1.0.5.tar.gz
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

## Pipe use...
```bash
$ echo "123 * 2" | dntk | xargs echo
246
```
behave, like bc.

***Have a nice go hacking days***:sparkles::wink:
## Writer & License
dntk was writed by nnao45 (WORK:Infrastructure Engineer, Twitter:@nnao45, MAIL:n4sekai5y@gmail.com).  
This software is released under the MIT License, see LICENSE.
