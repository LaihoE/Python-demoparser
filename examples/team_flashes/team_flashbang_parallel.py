from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    wanted_player = 76561198194694750
    # Only count as teamflash if >2s flashed. Small values are barely flashed
    min_blind_duration = 2

    parser = DemoParser(file)
    players = pd.DataFrame(parser.parse_players())
    players_blinded = pd.DataFrame(parser.parse_events("player_blind"))

    wanted_player_team = players[players["steamid"]
                                 == wanted_player]["starting_side"].values
    # Make sure player exists and has a team
    if len(wanted_player_team) > 0:
        wanted_player_team = wanted_player_team[0]
    else:
        return
    teammates = players[(players["starting_side"] == wanted_player_team) & (
        players["steamid"] != wanted_player)]
    teammates_steamids = teammates["steamid"].values

    players_blinded = players_blinded[(players_blinded["attacker_steamid"] == wanted_player) & (
        players_blinded["player_steamid"].isin(teammates_steamids))]
    players_blinded = players_blinded[players_blinded["blind_duration"]
                                      > min_blind_duration]
    return players_blinded


if __name__ == "__main__":
    files = glob.glob("/path/to/directory/with/demos/*")
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))

    df = pd.concat(results)
    print(df)
