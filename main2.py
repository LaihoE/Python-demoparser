from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os
from collections import Counter
import time


def coordinates(file):
    print(file)
    before = time.time()
    parser = DemoParser(file)
    #evs = pd.DataFrame(parser.parse_events("player_hurt"))
    df = parser.parse_ticks(["m_iClip1", "weapon_name"], ticks=[x for x in range(0,10000, 10)])
    #df = df[df["steamid"] == 76561198362125234]
    print(df.isna().sum())
    print(df)


if __name__ == "__main__":
    import time
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[:1]
    print(files)
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))

    df = pd.concat(results)
    df = df.groupby(["steamid"], sort=False)["delta"].mean().reset_index()

    print(df[df["steamid"] == 76561198134270402])
    print(df[df["steamid"] == 76561198048924300])
    print(df[df["steamid"] == 76561198194694750])
    print(df[df["steamid"] == 76561198066395116])
