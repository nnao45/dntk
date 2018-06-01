package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"

	"gopkg.in/alecthomas/kingpin.v2"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

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

const (
	COLOR_BLACK_HEADER   = "\x1b[30m"
	COLOR_RED_HEADER     = "\x1b[31m"
	COLOR_GREEN_HEADER   = "\x1b[32m"
	COLOR_YELLOW_HEADER  = "\x1b[33m"
	COLOR_BLUE_HEADER    = "\x1b[34m"
	COLOR_MAGENDA_HEADER = "\x1b[35m"
	COLOR_CYAN_HEADER    = "\x1b[36m"
	COLOR_WHITE_HEADER   = "\x1b[37m"
	COLOR_PLAIN_HEADER   = "\x1b[0m"
)

type pallet struct {
	Black   string
	Red     string
	Green   string
	Yellow  string
	Blue    string
	Magenda string
	Cyan    string
	White   string
	Plain   string
}

func newpallet() *pallet {
	return &pallet{
		Black:   COLOR_BLACK_HEADER,
		Red:     COLOR_RED_HEADER,
		Green:   COLOR_GREEN_HEADER,
		Yellow:  COLOR_YELLOW_HEADER,
		Blue:    COLOR_BLUE_HEADER,
		Magenda: COLOR_MAGENDA_HEADER,
		Cyan:    COLOR_CYAN_HEADER,
		White:   COLOR_WHITE_HEADER,
		Plain:   COLOR_PLAIN_HEADER,
	}
}

func (p *pallet) printMagenta(s string) string {
	return p.Magenda + fmt.Sprint(s) + p.Plain
}

func (p *pallet) printCyan(s string) string {
	return p.Cyan + fmt.Sprint(s) + p.Plain
}

var operator map[string]string = map[string]string{
	"+": "Plus",
	"-": "Minus",
	"*": "MultipliedBy",
	"/": "DividedBy",
}

type line struct {
	RuneByte []byte
	Buffer   []byte
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
	// TODO
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
	p := newpallet()

	for {

		os.Stdin.Read(l.RuneByte)

		if _, err := strconv.Atoi(string(l.RuneByte)); err != nil {
			// TODO
		} else if _, ok := operator[string(l.RuneByte)]; ok {
			// TODO
		}

		fmt.Print(p.printMagenta("\r" + strings.Repeat(" ", len(l.Buffer))))

		addog(fmt.Sprintln(l.RuneByte), "./test.txt")

		if fmt.Sprint(l.RuneByte) == "[127]" {
			l.Buffer = l.remove()
			fmt.Print(p.printMagenta("\r" + string(l.Buffer)))
			continue
		} else if string(l.RuneByte) == "q" || fmt.Sprint(l.RuneByte) == "[27]" {
			fmt.Print("\n")
			break
		}

		l.Buffer = append(l.Buffer, l.RuneByte...)
		fmt.Print(p.printMagenta("\r" + string(l.Buffer)))
	}
}
