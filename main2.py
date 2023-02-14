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
    print(file)
    before = time.time()
    parser = DemoParser(file)
    #df = parser.parse_ticks(["manager@m_iMatchStats_Damage_Total"], ticks=[x for x in range(9999998, 9999999)])
    #df["adr"] = df["manager@m_iMatchStats_Damage_Total"] / 26
    df = parser.parse_events("player_death")
    ticks = df["tick"].to_list()
    df = parser.parse_ticks(["player@m_vecOrigin_X"], ticks=ticks)
    print(df)
    #print(df)


if __name__ == "__main__":
    # files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[5:6]
    # files = glob.glob("/home/laiho/Documents/demos/mygames/*")[5:6]
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[240:]
    files = glob.glob("/home/laiho/Documents/demos/bench_pro_demos/*")
    before = time.time()
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files), desc="Parsing demos"))
    print(time.time() - before)


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