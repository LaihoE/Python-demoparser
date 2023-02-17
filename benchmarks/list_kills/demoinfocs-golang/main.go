package main

import (
	"fmt"
	"log"
	"os"

	dem "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs"
	events "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs/events"
)
type Kill struct{
	attacker string
	Victim string
}

func kill_events(demo_path string){
	f, err := os.Open(demo_path)
    if err != nil {
        log.Panic("failed to open demo file: ", err)
    }
    defer f.Close()
    p := dem.NewParser(f)
    defer p.Close()

	var players []Kill
	p.RegisterEventHandler(func(e events.Kill) {
		k := Kill{e.Killer.Name, e.Victim.Name}
		players = append(players, k)
	})
	err = p.ParseToEnd()
}


func main() {
	demo_dir := "/home/laiho/Documents/demos/bench_pro_demos/"
	demo_paths, err := os.ReadDir(demo_dir)
    if err != nil {
        log.Fatal(err)
    }
    for _, demo_name := range demo_paths {
		fmt.Println(demo_dir + demo_name.Name())
		kill_events(demo_dir + demo_name.Name())
	}
}