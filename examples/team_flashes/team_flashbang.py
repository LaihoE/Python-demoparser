from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


wanted_player = 76561198194694750
min_blind_duration = 2

parser = DemoParser("demo.dem")
players = pd.DataFrame(parser.parse_players())
players_blinded = pd.DataFrame(parser.parse_events("player_blind"))

wanted_player_team = players[players["steamid"]
                             == wanted_player]["starting_side"].values
# Make sure player exists and has a team
if len(wanted_player_team) > 0:
    wanted_player_team = wanted_player_team[0]
else:
    print("no team/player found")
    exit()
teammates = players[(players["starting_side"] == wanted_player_team) & (
    players["steamid"] != wanted_player)]
teammates_steamids = teammates["steamid"].values

players_blinded = players_blinded[(players_blinded["attacker_id"] == wanted_player) & (
    players_blinded["player_id"].isin(teammates_steamids))]
players_blinded = players_blinded[players_blinded["blind_duration"]
                                  > min_blind_duration]
                                  
print(players_blinded)
