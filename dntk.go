package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"golang.org/x/crypto/ssh/terminal"
	"gopkg.in/alecthomas/kingpin.v2"
)

var version string

var (
	app = kingpin.New("dntk", "This application is command line's Interactive calculator, GNU bc wrapper.")

	scale     = app.Flag("scale", "Number of truncated after the decimal point").Default("10").Short('s').Int()
	maxresult = app.Flag("maxresult", "Number of truncated after the result number").Default("999").Short('m').Int()
	unit      = app.Flag("unit", "Set the unit of result").Short('u').String()
	white     = app.Flag("white", "Set non color in a output").Default("false").Short('w').Bool()
	fixed     = app.Flag("fixed", "Add the fixed statement").Short('f').String()
	alias     = app.Flag("alias", "Add the custum variable").Short('a').String()
)

func sliceContains(str string, slice []string) bool {
	for _, v := range slice {
		if v == str {
			return true
		}
	}
	return false
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

func printYellow(s string) string {
	return COLOR_YELLOW_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

func printGreen(s string) string {
	return COLOR_GREEN_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

func printMagenta(s string) string {
	return COLOR_MAGENDA_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

func printCyan(s string) string {
	return COLOR_CYAN_HEADER + fmt.Sprint(s) + COLOR_PLAIN_HEADER
}

const (
	REBASE_KEY = "r"
	DELETE_KEY = "[127]"
)

const (
	SIN_FUNCTION_KEY      = "s" //s
	COS_FUNCTION_KEY      = "c" //c
	ATAN_FUNCTION_KEY     = "a" //a
	LOG_FUNCTION_KEY      = "l" //l
	EXP_FUNCTION_KEY      = "e" //e
	BESSEL_FUNCTION_KEY   = "j" //j
	RBRACKET_FUNCTION_KEY = "(" //(
)

var funcSlice []string = []string{
	SIN_FUNCTION_KEY,
	COS_FUNCTION_KEY,
	ATAN_FUNCTION_KEY,
	LOG_FUNCTION_KEY,
	EXP_FUNCTION_KEY,
	BESSEL_FUNCTION_KEY,
	RBRACKET_FUNCTION_KEY,
}

const (
	SINGLE_QUOTE_KEY = `'`
	DOUBLE_QUOTE_KEY = `"`
	BACK_QUOTE_KEY   = "`"
	BACK_SLASH_KEY   = `\`
	PIPE_KEY         = `|`
)

var dangerSlice []string = []string{
	SINGLE_QUOTE_KEY,
	DOUBLE_QUOTE_KEY,
	BACK_QUOTE_KEY,
	BACK_SLASH_KEY,
	PIPE_KEY,
}

const (
	Q_KEY      = "[113]"
	ENTER_KEY  = "[10]"
	ESCAPE_KEY = "[27]"
)

var killSlice []string = []string{
	Q_KEY,
	ENTER_KEY,
	ESCAPE_KEY,
}

type lineAlias struct {
	Alias string
	Value string
}

type line struct {
	RuneByte       []byte
	Buffer         []byte
	BufferAndEqual []byte
	Flag           bool
	Alert          bool
	FuncMode       bool
	FuncCounter    int
	LineAliasList  []lineAlias
}

func newline() *line {
	var r []byte = make([]byte, 1)
	var b []byte = make([]byte, 0)
	var e []byte = make([]byte, 0)
	return &line{
		RuneByte:       r,
		Buffer:         b,
		BufferAndEqual: e,
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
	if *white {
		return fmt.Sprint(s)
	}
	if l.Alert {
		return printYellow(s)
	}
	if l.FuncMode {
		return printGreen(s)
	}
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
	var fixedStr string
	if *fixed != "" {
		fixedStr = *fixed + " "
	}
	stdin := "echo \"scale=" + fmt.Sprint(*scale) + ";" + fixedStr + fmt.Sprint(string(trimSpaceFromByte(l.Buffer))) + "\" | bc -l"
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

func (l *line) printPrompt() {
	var unitStr string
	if *unit != "" {
		unitStr = "(" + *unit + ")"
	}
	var fixedStr string
	if *fixed != "" {
		fixedStr = *fixed + " "
	}
	l.BufferAndEqual = append(append(append(append([]byte("(dntk): "), []byte(fixedStr)...), append(l.Buffer, []byte(" = ")...)...), l.calcBuffer()...), []byte(unitStr)...)
	fmt.Print(l.dntkPrint("\r" + string(l.BufferAndEqual)))
}

func (l *line) appnedBuffer() {
	var x int
	for _, v := range l.LineAliasList {
		if string(l.RuneByte) == v.Alias {
			l.Buffer = append(l.Buffer, []byte(v.Value)...)
			x++
		}
	}
	if x != 0 {
		return
	}
	l.Buffer = append(l.Buffer, l.RuneByte...)

}

func (l *line) printBuffer() {
	if fmt.Sprint(l.RuneByte) != DELETE_KEY {
		l.appnedBuffer()
	}

	l.printPrompt()
}

func (l *line) printFuncBuffer() {
	if string(l.RuneByte) != "(" {
		l.appnedBuffer()
	}
	l.Buffer = append(l.Buffer, []byte("(")...)
	l.FuncCounter++

	l.printPrompt()
}

func (l *line) printFuncQuitBuffer() {
	l.Buffer = append(l.Buffer, []byte(")")...)
	l.FuncCounter--

	l.printPrompt()
	if l.FuncCounter < 1 {
		l.FuncMode = false
	}
}

func (l *line) printAlert() {
	l.Alert = true
	alertString := fmt.Sprintf("%v \"%v\" %v", "Sorry,", string(l.RuneByte), "is Danger word. Please not use.")
	fmt.Print(l.dntkPrint("\r" + alertString))
	l.Alert = false
	time.Sleep(time.Second)
	fmt.Print(l.dntkPrint("\r" + strings.Repeat(" ", len(alertString))))
	l.printPrompt()
}

func (l *line) parseAliasOpt() {
	var runeSlice []rune
	var runeSilceList [][]rune
	for _, r := range *alias {
		if r == ',' {
			runeSilceList = append(runeSilceList, runeSlice)
			runeSlice = []rune{}
		} else {
			runeSlice = append(runeSlice, r)
		}
	}
	runeSilceList = append(runeSilceList, runeSlice)
	for _, rs := range runeSilceList {
		l.appendLineAliasList(rs)
	}
}

func (l *line) appendLineAliasList(rs []rune) {
	var vari, valu []rune
	var afterEqrual bool
	for _, r := range rs {
		if r == ' ' {
			continue
		}
		if afterEqrual {
			valu = append(valu, r)
		} else {
			if r == '=' {
				afterEqrual = true
				continue
			}
			vari = append(vari, r)
		}
	}
	fValu, err := strconv.ParseFloat(string(valu), 64)
	if err != nil {
		fmt.Fprintln(os.Stderr, fmt.Sprintf("%v \"%v\" %v", "Sorry,", string(valu), "is not number. Please not use."))
		os.Exit(1)
	}

	l.LineAliasList = append(l.LineAliasList, lineAlias{
		Alias: string(vari),
		Value: fmt.Sprint(fValu),
	})
	//fmt.Println(l.LineVariable.Varialbe)
}

/*
func (l *line) funcJudge() {
	var funcCount int
	var bracketCount int
	var k int
	for _, b := range string(l.Buffer) {
		if sliceContains(string(b), funcSlice) {
			funcCount++
		}
	}
	for i, b := range string(l.Buffer) {
		if string(b) == "(" {
			k = i
		}
		if string(b) == ")" && k < i {
			bracketCount++
		}
	}
	if funcCount == bracketCount {
		l.FuncMode = true
	}
}*/

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	// TODO
	}

	os.Setenv("BC_LINE_LENGTH", fmt.Sprint(*maxresult))
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
	if *alias != "" {
		l.parseAliasOpt()
	}

	if !terminal.IsTerminal(0) {
		for {
			os.Stdin.Read(l.RuneByte)
			if sliceContains(string(l.RuneByte), dangerSlice) {
				fmt.Fprintln(os.Stderr, fmt.Sprintf("%v \"%v\" %v", "Sorry,", string(l.RuneByte), "is Danger word. Please not use."))
				os.Exit(1)
			} else if fmt.Sprint(l.RuneByte) == "[10]" {
				break
			}
			l.Buffer = append(l.Buffer, l.RuneByte...)
		}
		fmt.Println(string(l.calcBuffer()))
		return
	}

	var fixedStr string
	if *fixed != "" {
		fixedStr = *fixed + " "
	}
	fmt.Print(l.dntkPrint("\r" + string([]byte("(dntk): ")) + fixedStr))
	for {

		os.Stdin.Read(l.RuneByte)

		fmt.Print(l.dntkPrint("\r" + strings.Repeat(" ", len(l.BufferAndEqual))))

		if len(l.Buffer) < 1 {
			l.Flag = false
			l.FuncMode = false
			l.FuncCounter = 0
		}

		if string(l.RuneByte) == REBASE_KEY {
			l = newline()
			if *alias != "" {
				l.parseAliasOpt()
			}
			fmt.Print(l.dntkPrint("\r" + string([]byte("(dntk): "))))
			continue
		} else if fmt.Sprint(l.RuneByte) == DELETE_KEY {
			// send delete key OR backspace key
			l.Buffer = l.remove()
			l.printBuffer()
			continue
		} else if sliceContains(string(l.RuneByte), dangerSlice) {
			l.printAlert()
			continue
		} else if sliceContains(fmt.Sprint(l.RuneByte), killSlice) {
			// send "q" key OR escape key OR Enter key
			if l.FuncMode {
				l.printFuncQuitBuffer()
				continue
			}
			fmt.Println(l.dntkPrint("\r" + string(l.BufferAndEqual)))
			break
		} else if sliceContains(string(l.RuneByte), funcSlice) {
			l.FuncMode = true
			l.printFuncBuffer()
			continue
		}

		l.printBuffer()
	}
}
