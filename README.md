[![Travis CI](https://travis-ci.org/nnao45/dntk.svg?branch=master)](https://travis-ci.org/nnao45/dntk)
[![v2.0.0](https://img.shields.io/badge/package-v2.0.0-ff69b4.svg)](https://github.com/nnao45/dntk/releases/tag/v2.0.0)
[![license](http://img.shields.io/badge/license-MIT-red.svg?style=flat)](https://raw.githubusercontent.com/nnao45/dntk/master/LICENSE)
[![Go Report Card](https://goreportcard.com/badge/github.com/nnao45/dntk)](https://goreportcard.com/report/github.com/nnao45/dntk)

# dntk
dntk is command line's ***Interactive*** calculator, [GNU bc](https://www.gnu.org/software/bc/) wrapper.  
![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-demo01.gif)
  
✔︎ dntk means calculator in a japanese.  
✔︎ dntk is gnu bc wrapper. so, syntax is equal to gnu bc. [learn syntax more](https://www.gnu.org/software/bc/manual/html_mono/bc.html)  
✔︎ dntk is a NATIVE [The Go Programming Language](http://golang.org) application.  
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
        <td>input function mode</td>
    </tr>
</table>

## ***Current dntk's version: v2.0.0***
Download Page: https://github.com/nnao45/dntk/releases/latest

## Install
### Mac
```bash
$ brew install nnao45/dntk/dntk
```

### Linux
```bash
$ wget https://github.com/nnao45/dntk/releases/download/v2.0.0/dntk-linux-amd64-v2.0.0.tar.gz
$ tar xvfz dntk-linux-amd64-v2.0.0.tar.gz
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

## Option

```
$ dntk -h
usage: dntk [<flags>]

This application is command line's Interactive calculator, GNU bc wrapper.

Flags:
  -h, --help           Show context-sensitive help (also try --help-long and
                       --help-man).
  -s, --scale=10       Number of truncated after the decimal point
  -m, --maxresult=999  Number of truncated after the result number
  -u, --unit=UNIT      Set the unit of result
  -w, --white          Set non color in a output
  -f, --fixed=FIXED    Add the fixed statement
  -a, --alias=ALIAS    Add the custum alias
      --version        Show application version.
```

## function input easy
dntk can use function.

![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-demo02.gif)

you can use under function.

<table>
    <tr>
        <td>function</td>
        <td>command</td>
        <td>detail</td>
    </tr>
    <tr>
        <td>(x)</td>
        <td>(</td>
        <td>Simple round bracket</td>
    </tr>
    <tr>
        <td>sin(x)</td>
        <td>s</td>
        <td>Sin of trigonometric function</td>
    </tr>
    <tr>
        <td>cos(x)</td>
        <td>c</td>
        <td>Cosin of trigonometric function</td>
    </tr>
    <tr>
        <td>atan(x)</td>
        <td>a</td>
        <td>Tangent of inverse trigonometric function</td>
    </tr>
    <tr>
        <td>log(x)</td>
        <td>l</td>
        <td>Logarithm function</td>
    </tr>
    <tr>
        <td>exp(x)</td>
        <td>e</td>
        <td>Exponential function</td>
    </tr>
    <tr>
        <td>j(n,x)</td>
        <td>j</td>
        <td>The n-order Bessel function</td>
    </tr>
</table>

example, if you want to write `(123 + 2) * 3`, you type,

```
(
1
2
3
+
2
Enter
*
3
Enter
```

if you want to write `a(123)`, you type,

```
a
1
2
3
Enter
Enter
```

if you want to write`a(123) * c(678 * 123)`, you type,

```
a
1
2
3
Enter
*
c
6
7
8
*
1
2
3
Enter
Enter
```

very easy 😘

## set fixed value
example, excange calculate AWS Billing, JPY -> USD, put fixed statement

![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-demo04.gif)

## set alias
you can use alias

```
$ dntk -a '<alias char>=<value>,<alias char>=<value>,...'
```

![result](https://github.com/nnao45/naoGifRepo/blob/master/dntk-demo03.gif)

☝️ type,

```
x
*
y
*
y
```

you can write long long value very easy 😁

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
