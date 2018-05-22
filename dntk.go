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

//func bufferLine

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

//func keyHook(devs []*keylogger.InputDevice) {
//	for _, val := range devs {
//		fmt.Println("Id->", val.Id, "Device->", val.Name)
//	}
//}

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	}
}

func main() {
	ttyctl()

	l := newline()

	//devs, err := keylogger.NewDevices()
	//if err != nil {
	//	panic(err)
	//}

	//go keyHook(devs)

	for {
		os.Stdin.Read(l.RuneByte)
		l.Buffer = append(l.Buffer, l.RuneByte...)
		//if string(b.RuneByte) == "" {

		//}
		fmt.Print("\r", string(l.Buffer))
	}
}
