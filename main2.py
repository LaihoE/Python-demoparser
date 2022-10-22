from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os
from collections import Counter


def coordinates(file):
    parser = DemoParser(file)
    df = parser.parse_ticks(["m_iLifetimeStart", "m_iLifetimeEnd"])
    df = df.dropna()
    df = df[df["m_iLifetimeEnd"] != -1]
    df["delta"] = df["m_iLifetimeEnd"] - df["m_iLifetimeStart"] -15
    return df


if __name__ == "__main__":
    import time
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))

    df = pd.concat(results)
    df = df.groupby(["steamid"], sort=False)["delta"].mean().reset_index()
    
    print(df[df["steamid"] == 76561198134270402])
    print(df[df["steamid"] == 76561198048924300])
    print(df[df["steamid"] == 76561198194694750])
    print(df[df["steamid"] == 76561198066395116])
