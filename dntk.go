package main

import (
	"fmt"
	"gopkg.in/alecthomas/kingpin.v2"
	"io/ioutil"
	"os"
	"syscall"

	"golang.org/x/crypto/ssh/terminal"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("dntk's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	}
}

func main() {
	if terminal.IsTerminal(syscall.Stdin) {
		// Execute: go run main.go
		fmt.Print("Type something then press the enter key: ")
		var stdin string
		fmt.Scan(&stdin)
		fmt.Printf("Result: %s\n", stdin)
		return
	}

	// Execute: echo "foo" | go run main.go
	body, err := ioutil.ReadAll(os.Stdin)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Result: %s\n", string(body))
}
