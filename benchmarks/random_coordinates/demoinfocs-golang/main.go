package main

import (
	"fmt"
	"log"
	"os"

	dem "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs"
	events "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs/events"
)
type Coordinate struct{
	name string
	tick int
	x float64
	y float64
	z float64
}

func kill_events(demo_path string){
	f, err := os.Open(demo_path)
    if err != nil {
        log.Panic("failed to open demo file: ", err)
    }
    defer f.Close()
    p := dem.NewParser(f)
    defer p.Close()

	// 3m48,336s

	var c []Coordinate

	m := make(map[int]bool)
	for i := 0; i < 100; i++{
		m[i * 10000] = true
	}
	p.RegisterEventHandler(func(e events.FrameDone) {
		if _, ok := m[p.GameState().IngameTick()]; ok {
			for _, player := range p.GameState().Participants().Playing() {
				k := Coordinate{
					player.Name,
					p.GameState().IngameTick(),
					player.LastAlivePosition.X,
					player.LastAlivePosition.Y,
					player.LastAlivePosition.Z}
				//fmt.Println(k)
				c = append(c, k)
			}
		}

	})
	err = p.ParseToEnd()
	//fmt.Println(c)

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