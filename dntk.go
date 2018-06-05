package main

import (
	"bufio"
	"bytes"
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

func printMagenta(s string) string {
	return COLOR_MAGENDA_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

func printCyan(s string) string {
	return COLOR_CYAN_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

const (
	SIN_FUNCTION_KEY      = "[115]" //s
	COS_FUNCTION_KEY      = "[99]"  //c
	ATAN_FUNCTION_KEY     = "[97]"  //a
	LOG_FUNCTION_KEY      = "[108]" //l
	EXP_FUNCTION_KEY      = "[101]" //e
	BESSEL_FUNCTION_KEY   = "[106]" //j
	RBRACKET_FUNCTION_KEY = "[40]"  //(
)

var funcMap map[string]string = map[string]string{
	SIN_FUNCTION_KEY:      "SIN_FUNCTION_KEY",
	COS_FUNCTION_KEY:      "COS_FUNCTION_KEY",
	ATAN_FUNCTION_KEY:     "ATAN_FUNCTION_KEY",
	LOG_FUNCTION_KEY:      "LOG_FUNCTION_KEY",
	EXP_FUNCTION_KEY:      "EXP_FUNCTION_KEY",
	BESSEL_FUNCTION_KEY:   "BESSEL_FUNCTION_KEY",
	RBRACKET_FUNCTION_KEY: "RBRACKET_FUNCTION_KEY",
}

type line struct {
	RuneByte       []byte
	Buffer         []byte
	BufferAndEqual []byte
	Flag           bool
	FuncMode       bool
}

func newline() *line {
	var r []byte = make([]byte, 1)
	var l []byte = make([]byte, 0)
	return &line{
		RuneByte: r,
		Buffer:   l,
	}
}

func (l *line) remove() (bary []byte) {
	for i, b := range l.Buffer {
		if i == len(l.Buffer)-1 {
			break
		}
		bary = append(bary, b)
	}

	return
}

func (l *line) dntkPrint(s string) string {
	if l.Flag {
		return printCyan(s)
	}
	return printMagenta(s)
}

func (l *line) appendEqual() []byte {
	l.Buffer = append(l.Buffer, []byte(" = ")...)
	return l.Buffer
}

func trimSpaceFromByte(s []byte) (byt []byte) {
	for _, b := range s {
		if string(b) == " " {
			continue
		}
		byt = append(byt, b)
	}
	return
}

func (l *line) calcBuffer() []byte {
	var stdout, stderr bytes.Buffer
	l.Flag = true
	stdin := "echo \"scale=10;" + fmt.Sprint(string(trimSpaceFromByte(l.Buffer))) + "\" | bc -l"
	cmd := exec.Command("sh", "-c", stdin)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	err := cmd.Run()
	if err != nil {
		panic(err)
	}
	var result []byte

	for i, r := range stdout.Bytes() {
		if i == len(stdout.Bytes())-1 {
			break
		}
		result = append(result, r)
	}
	if len(stderr.Bytes()) > 0 {
		l.Flag = false
		result = []byte("nil")
	}
	return result
}

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	// TODO
	}

	os.Setenv("BC_LINE_LENGTH", "999")
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
	var result []byte

	fmt.Print(l.dntkPrint("\r" + string([]byte("(dntk): "))))

	for {

		os.Stdin.Read(l.RuneByte)
		fmt.Print(l.dntkPrint("\r" + strings.Repeat(" ", len(l.BufferAndEqual))))

		addog(fmt.Sprintln(l.RuneByte), "./test.txt")

		if l.FuncMode {
			//TODO
		}

		if fmt.Sprint(l.RuneByte) == "[127]" {
			// send delete key OR backspace key
			l.Buffer = l.remove()
			result = l.calcBuffer()
			l.BufferAndEqual = append(append([]byte("(dntk): "), append(l.Buffer, []byte(" = ")...)...), result...)
			fmt.Print(l.dntkPrint("\r" + string(l.BufferAndEqual)))
			continue
		} else if string(l.RuneByte) == "q" || fmt.Sprint(l.RuneByte) == "[27]" || fmt.Sprint(l.RuneByte) == "[10]" {
			// send "q" key OR escape key OR Enter key
			fmt.Println("\r" + string(l.BufferAndEqual))
			break
		} else if _, ok := funcMap[string(l.RuneByte)]; ok {
			//TODO
			if string(l.RuneByte) != "[40]" {
				l.Buffer = append(l.Buffer, l.RuneByte...)
			}
			l.FuncMode = true
			l.BufferAndEqual = append(append([]byte("(dntk): "), append(l.Buffer, []byte(" = ")...)...), result...)
			continue
		} else if string(l.RuneByte) == "[9]" {
			// send tab key
			// TODO
			fmt.Print("\n")
			break
		}

		l.Buffer = append(l.Buffer, l.RuneByte...)
		result = l.calcBuffer()
		l.BufferAndEqual = append(append([]byte("(dntk): "), append(l.Buffer, []byte(" = ")...)...), result...)
		fmt.Print(l.dntkPrint("\r" + string(l.BufferAndEqual)))
	}
}
