from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os
from collections import Counter


def coordinates(file):
    time.sleep(1)
    parser = DemoParser(file)
    #parser.parse_events("")
    df = parser.parse_ticks(["health", "#adfsg", "adfg"])
    #print(df)



if __name__ == "__main__":
    import time
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    before = time.time()
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))

    df = pd.concat(results)
    df = df.groupby(["steamid"], sort=False)["delta"].mean().reset_index()

    print(df[df["steamid"] == 76561198134270402])
    print(df[df["steamid"] == 76561198048924300])
    print(df[df["steamid"] == 76561198194694750])
    print(df[df["steamid"] == 76561198066395116])
