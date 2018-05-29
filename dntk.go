package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"gopkg.in/alecthomas/kingpin.v2"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

type line struct {
	RuneByte []byte
	Buffer   []byte
}

func addog(text string, filename string) {
	var writer *bufio.Writer
	textData := []byte(text)

	writeFile, err := os.OpenFile(filename, os.O_CREATE|os.O_APPEND|os.O_RDWR, 0755)
	writer = bufio.NewWriter(writeFile)
	writer.Write(textData)
	writer.Flush()
	if err != nil {
		panic(err)
	}
	defer writeFile.Close()
}

func newline() *line {
	var r []byte = make([]byte, 1)
	var l []byte = make([]byte, 0)
	return &line{
		RuneByte: r,
		Buffer:   l,
	}
}

func (l *line) remove() (ary []byte) {
	for i, b := range l.Buffer {
		if i == len(l.Buffer)-1 {
			break
		}
		ary = append(ary, b)
	}
	return ary
}

func (l *line) calcBuffer() {
}

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	//
	}
}

func main() {
	// disable input buffering
	exec.Command("stty", "-F", "/dev/tty", "cbreak", "min", "1").Run()
	// delete \n
	exec.Command("stty", "-F", "/dev/tty", "erase", "\n").Run()
	// do not display entered characters on the screen
	exec.Command("stty", "-F", "/dev/tty", "-echo").Run()
	// restore the echoing state when exiting
	defer exec.Command("stty", "-F", "/dev/tty", "echo").Run()

	l := newline()

	for {

		os.Stdin.Read(l.RuneByte)
		fmt.Print("\r", strings.Repeat(" ", len(l.Buffer)))

		addog(fmt.Sprintln(l.RuneByte), "./test.txt")

		if fmt.Sprint(l.RuneByte) == "[127]" {
			l.Buffer = l.remove()
			fmt.Print("\r", string(l.Buffer))
			continue
		} else if string(l.RuneByte) == "q" || fmt.Sprint(l.RuneByte) == "[27]" {
			fmt.Print("\n")
			break
		}

		l.Buffer = append(l.Buffer, l.RuneByte...)
		fmt.Print("\r", string(l.Buffer))
	}
}
