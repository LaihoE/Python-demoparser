package main

import (
	"fmt"
	"log"
	"os"
	"encoding/csv"

	dem "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs"
	events "github.com/markus-wa/demoinfocs-golang/v3/pkg/demoinfocs/events"
)


func main() {
	f, err := os.Open("/home/laiho/Documents/demos/faceits/cu/1-2ece7f36-87b9-4723-9c61-ca90c85c02a7_76561197992979809.dem")
	//f, err := os.Open("/home/laiho/Documents/demos/mygames/1.dem")
	if err != nil {
		log.Panic("failed to open demo file: ", err)
	}
	defer f.Close()
	p := dem.NewParser(f)
	defer p.Close()

	ff, err := os.Create("killevent.csv")
    defer ff.Close()

    if err != nil {

        log.Fatalln("failed to open file", err)
    }

	
    w := csv.NewWriter(ff)
    defer w.Flush()

	all := [][]string{}

	// did I just do that?
	labels := []string{"attacker_X","attacker_Y","attacker_Z","attacker_flash_duration", "attacker_velocity_X", "attacker_velocity_Y",
	"attacker_velocity_Z", "attacker_viewangle_yaw", "attacker_viewangle_pitch",
	"attacker_userid", "attacker_steamid", "attacker_entityid", "attacker_team_num", "attacker_assists", "attacker_crosshair_code",
	"attacker_deaths", "attacker_total_equipment_value", "attacker_freeze_end_eq_val", "attacker_health", "attacker_scoped",
	"attacker_kills", "attacker_mvps", "attacker_balance", "attacker_ping", "attacker_score", "attacker_ammo", "attacker_name", "attacker_weapon_name", "attacker_place_name",
	
	"player_X","player_Y","player_Z","player_flash_duration", "player_velocity_X", "player_velocity_Y",
	"player_velocity_Z", "player_viewangle_yaw", "player_viewangle_pitch",
	"player_userid", "player_steamid", "player_entityid", "player_team_num", "player_assists", "player_crosshair_code",
	"player_deaths", "player_total_equipment_value", "player_freeze_end_eq_val", "player_health", "player_scoped",
	"player_kills", "player_mvps", "player_balance", "player_ping", "player_score", "player_ammo", "player_name", "player_weapon_name","player_place_name",
	}
	all = append(all, labels)

	p.RegisterEventHandler(func(e events.Kill) {
		// Yes I just did that
		var out []string
		out = append(out, fmt.Sprintf("%.3f",e.Killer.LastAlivePosition.X))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.LastAlivePosition.Y))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.LastAlivePosition.Z))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.FlashDuration))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.Velocity().X))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.Velocity().Y))
		out = append(out, fmt.Sprintf("%.3f",e.Killer.Velocity().Z))
		out = append(out, fmt.Sprintf("%.3f", e.Killer.ViewDirectionX()))
		out = append(out, fmt.Sprintf("%.3f", e.Killer.ViewDirectionY()))
		out = append(out, fmt.Sprintf("%d", e.Killer.UserID))
		out = append(out, fmt.Sprintf("%d", e.Killer.SteamID64))
		out = append(out, fmt.Sprintf("%d", e.Killer.EntityID))
		out = append(out, fmt.Sprintf("%d", e.Killer.Team))
		out = append(out, fmt.Sprintf("%d", e.Killer.Assists()))
		out = append(out, e.Killer.CrosshairCode())
		out = append(out, fmt.Sprintf("%d", e.Killer.Deaths()))
		out = append(out, fmt.Sprintf("%d", e.Killer.EquipmentValueCurrent()))
		out = append(out, fmt.Sprintf("%d", e.Killer.EquipmentValueFreezeTimeEnd()))
		out = append(out, fmt.Sprintf("%d", e.Killer.Health()))
		out = append(out, fmt.Sprintf("%t", e.Killer.IsScoped()))
		out = append(out, fmt.Sprintf("%d", e.Killer.Kills()))
		out = append(out, fmt.Sprintf("%d", e.Killer.MVPs()))
		out = append(out, fmt.Sprintf("%d", e.Killer.Money()))
		out = append(out, fmt.Sprintf("%d", e.Killer.Ping()))
		out = append(out, fmt.Sprintf("%d", e.Killer.Score()))
		out = append(out, fmt.Sprintf("%d",e.Killer.ActiveWeapon().AmmoInMagazine()))
		out = append(out, fmt.Sprintf(e.Killer.Name))
		out = append(out, fmt.Sprintf(e.Killer.ActiveWeapon().String()))
		out = append(out, fmt.Sprintf(e.Killer.LastPlaceName()))

		out = append(out, fmt.Sprintf("%.3f",e.Victim.LastAlivePosition.X))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.LastAlivePosition.Y))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.LastAlivePosition.Z))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.FlashDuration))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.Velocity().X))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.Velocity().Y))
		out = append(out, fmt.Sprintf("%.3f",e.Victim.Velocity().Z))
		out = append(out, fmt.Sprintf("%.3f", e.Victim.ViewDirectionX()))
		out = append(out, fmt.Sprintf("%.3f", e.Victim.ViewDirectionY()))
		out = append(out, fmt.Sprintf("%d", e.Victim.UserID))
		out = append(out, fmt.Sprintf("%d", e.Victim.SteamID64))
		out = append(out, fmt.Sprintf("%d", e.Victim.EntityID))
		out = append(out, fmt.Sprintf("%d", e.Victim.Team))
		out = append(out, fmt.Sprintf("%d", e.Victim.Assists()))
		out = append(out, e.Victim.CrosshairCode())
		out = append(out, fmt.Sprintf("%d", e.Victim.Deaths()))
		out = append(out, fmt.Sprintf("%d", e.Victim.EquipmentValueCurrent()))
		out = append(out, fmt.Sprintf("%d", e.Victim.EquipmentValueFreezeTimeEnd()))
		out = append(out, fmt.Sprintf("%d", e.Victim.Health()))
		out = append(out, fmt.Sprintf("%t", e.Victim.IsScoped()))
		out = append(out, fmt.Sprintf("%d", e.Victim.Kills()))
		out = append(out, fmt.Sprintf("%d", e.Victim.MVPs()))
		out = append(out, fmt.Sprintf("%d", e.Victim.Money()))
		out = append(out, fmt.Sprintf("%d", e.Victim.Ping()))
		out = append(out, fmt.Sprintf("%d", e.Victim.Score()))
		out = append(out, fmt.Sprintf("%d",e.Victim.ActiveWeapon().AmmoInMagazine()))
		out = append(out, fmt.Sprintf(e.Victim.Name))
		out = append(out, fmt.Sprintf(e.Victim.ActiveWeapon().String()))
		out = append(out, fmt.Sprintf(e.Victim.LastPlaceName()))
		all = append(all, out)
	})

	
	// Parse to end
	err = p.ParseToEnd()
	if err != nil {
		log.Panic("failed to parse demo: ", err)
	}

	for _, row := range all {
		_ = w.Write(row)
	}

}