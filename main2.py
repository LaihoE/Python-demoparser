from demoparser import DemoParser
import pandas as pd
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import time


def coordinates(file):
    print(file)
    parser = DemoParser(file)
    before = time.time()
    df = pd.DataFrame(parser.parse_events_fast(
        "player_death", props=["X", "Y", "velocity_X", "velocity_Y"]))
    # print(pd.DataFrame(parser.parse_players()))
    #print("ticks", parser.parse_header()["playback_frames"])
    #print(time.time() - before)
    # print(df["attacker_m_vecOrigin_X"])
    # print(df["player_m_iHealth"])
    # print(df.columns)
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")  # [:100]

    with mp.Pool(processes=24) as pool:
        results = list(pool.map(coordinates, files))
    df = pd.concat(results)
    print(df)

    df = df[df["player_steamid"] != 0]
    df = df[df["attacker_steamid"] != 0]

    print(df.isna().sum())
    # print(Counter(df["attacker_m_bSpotted"]))
