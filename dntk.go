package main

import (
	"fmt"
	"gopkg.in/alecthomas/kingpin.v2"
	"os"
	"os/exec"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

type bufferLine struct {
	RuneByte   []byte
	LineBuffer []byte
}

func newbufferLine() *bufferLine {
	var r []byte = make([]byte, 1)
	var l []byte = make([]byte, 32)
	return &bufferLine{
		RuneByte:   r,
		LineBuffer: l,
	}
}

func ttyctl() {
	// disable input buffering
	exec.Command("stty", "-F", "/dev/tty", "cbreak", "min", "1").Run()
	// delete \n
	exec.Command("stty", "-F", "/dev/tty", "erase", "\n")
	// do not display entered characters on the screen
	exec.Command("stty", "-F", "/dev/tty", "-echo").Run()
	// restore the echoing state when exiting
	defer exec.Command("stty", "-F", "/dev/tty", "echo").Run()
}

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	}
}

func main() {
	ttyctl()

	b := newbufferLine()
	for {
		os.Stdin.Read(b.RuneByte)
		b.LineBuffer = append(b.LineBuffer, b.RuneByte...)
		fmt.Print("\r", string(b.LineBuffer))
	}
}
