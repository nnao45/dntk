package main

import (
	"fmt"
	"gopkg.in/alecthomas/kingpin.v2"
	"os"
)

var version string

var (
	app = kingpin.New("dntk", "A dntk application.")
)

func init() {
	app.HelpFlag.Short('h')
	app.Version(fmt.Sprint("baketsu's version: ", version))
	switch kingpin.MustParse(app.Parse(os.Args[1:])) {
	}
}

func main() {
	fmt.Println("vim-go")
}
