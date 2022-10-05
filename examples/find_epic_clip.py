import multiprocessing
import glob
import pandas as pd
from demoparser import DemoParser


def find_kills(file):
    parser = DemoParser(file)
    game_events = parser.parse_events("player_death")
    df = pd.DataFrame(game_events)
    df = df[df["attacker_name"] == "flusha"]
    return df


if __name__ == "__main__":
    files = glob.glob("/directory/with/demos/*")
    with multiprocessing.Pool(processes=12) as pool:
        results = pool.map(find_kills, files)
