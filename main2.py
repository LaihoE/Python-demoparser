from demoparser import DemoParser
import pandas as pd
import glob
import multiprocessing as mp
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt
import numpy as np


def coordinates(file):
    if file == "/home/laiho/Documents/demos/faceits/cu/003309131115255562271_1824323488 (1).dem":
        return

    parser = DemoParser(file)
    #df = pd.DataFrame(parser.parse_events("player_death"))
    print(parser.parse_players())

    # print(df.columns)
    # print(df.loc[:, ["name", "weapid", "tick", "Ammo", "player@m_vecOrigin_X"]])

if __name__ == "__main__":
    # files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[5:60]
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")[22:23]
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[240:]
    # files = glob.glob("/home/laiho/Documents/demos/bench_pro_demos/*")[:20]

    before = time.time()
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files), desc="Parsing demos"))
    
    df = pd.concat(results)
    df = df[(df["distance"] > 30) & (df["noscope"] == True)]
    print(df)


"""
0  76561197993611582  10000                           50.0
1  76561198089780719  10000                          450.0
2  76561198134270402  10000                          150.0
3  76561198147100782  10000                          150.0
4  76561198189734257  10000                          150.0
5  76561198194694750  10000                          150.0
6  76561198201296319  10000                          400.0
7  76561198229793868  10000                          150.0
8  76561198258044111  10000                         1100.0
9  76561198271657717  10000                          750.0
"""