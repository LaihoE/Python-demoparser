from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os
from collections import Counter


def coordinates(file):
    print(file)
    time.sleep(1)
    parser = DemoParser(file)
    evs = pd.DataFrame(parser.parse_events("player_hurt"))
    players = pd.DataFrame(parser.parse_players())
    print(evs["round"].max())

    """
    df = parser.parse_ticks(["m_iClip1", "weapon_name"])
    df = df[df["steamid"] == 76561198194694750]
    for i in range(len(df)):
        print(df.iloc[i].to_list())
    print(set(df["weapon_name"]))
    print(df.isna().sum(), len(df))
    """



if __name__ == "__main__":
    import time
    files = glob.glob("/mnt/d/b/mygames/*")
    print(files)
    before = time.time()
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))

    df = pd.concat(results)
    df = df.groupby(["steamid"], sort=False)["delta"].mean().reset_index()

    print(df[df["steamid"] == 76561198134270402])
    print(df[df["steamid"] == 76561198048924300])
    print(df[df["steamid"] == 76561198194694750])
    print(df[df["steamid"] == 76561198066395116])
