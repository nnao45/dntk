package main

import (
	"fmt"
	//	"github.com/MarinX/keylogger"
	"gopkg.in/alecthomas/kingpin.v2"
	"os"
	"os/exec"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

type line struct {
	RuneByte []byte
	Buffer   []byte
}

func newline() *line {
	var r []byte = make([]byte, 1)
	var l []byte = make([]byte, 32)
	return &line{
		RuneByte: r,
		Buffer:   l,
	}
}

func (l *line) remove() {
	var ary []byte = make([]byte, 32)
	for i, b := range l.Buffer {
		if i == len(l.Buffer)-1 {
			break
		}
		ary = append(ary, b)
	}
	copy(ary, l.Buffer)
}
func ttyctl() {
	// disable input buffering
	exec.Command("stty", "-F", "/dev/tty", "cbreak", "min", "1").Run()
	// delete \n
	exec.Command("stty", "-F", "/dev/tty", "erase", "\n").Run()
	// do not display entered characters on the screen
	exec.Command("stty", "-F", "/dev/tty", "-echo").Run()
	// restore the echoing state when exiting
	defer exec.Command("stty", "-F", "/dev/tty", "echo").Run()
}

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	//
	}
}

func main() {
	ttyctl()

	l := newline()

	for {
		os.Stdin.Read(l.RuneByte)
		l.Buffer = append(l.Buffer, l.RuneByte...)

		if string(l.RuneByte) == "\b" {
			l.remove()
		}

		fmt.Print("\r", string(l.Buffer))
	}
}
