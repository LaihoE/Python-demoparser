from demoparser import DemoParser
import pandas as pd
import glob
import multiprocessing as mp
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt
import numpy as np
from pandas.testing import assert_frame_equal

def coordinates(file):
    #parser = DemoParser("/home/laiho/Documents/programming/rust/newparser/Python-demoparser/tests/test_demo.dem")
    parser = DemoParser(file)
    df = parser.parse_ticks(["X"], ticks=[x for x in range(30000, 30001)])
    #df = parser.parse_events("player_death")
    return df
    #print(df)
    """df = parser.parse_ticks(["weapon", "ammo"], ticks=[x for x in range(22000, 22001)])
    df = df[df["tick"] == 22000]
    correct = {'tick': {132000: 22000, 132001: 22000, 132002: 22000, 132003: 22000, 132004: 22000, 132005: 22000, 132006: 22000, 132007: 22000, 132008: 22000, 132009: 22000, 132010: 22000}, 'name': {132000: 'Bert', 132001: 'mormoncrew', 132002: 'NN', 132003: 'KosatkaPeek mosambitchpeek', 132004: 'Bewer', 132005: 'Ganterhatced', 132006: 'Road to LE', 132007: 'duck', 132008: 'Psychopath', 132009: '-ExΩtiC-', 132010: 'krYtep'}, 'steamid': {132000: 0, 132001: 76561197993611582, 132002: 76561198089780719, 132003: 76561198134270402, 132004: 76561198147100782, 132005: 76561198189734257, 132006: 76561198194694750, 132007: 76561198201296319, 132008: 76561198229793868, 132009: 76561198258044111, 132010: 76561198271657717}, 'weapon': {132000: 'BaseWeaponWorldModel', 132001: 'molotov', 132002: 'awp', 132003: 'bizon', 132004: 'm4a1_silencer', 132005: 'p90', 132006: 'ak47', 132007: 'm4a4', 132008: 'awp', 132009: None, 132010: 'ak47'}, 'ammo': {132000: np.nan, 132001: -1.0, 132002: 10.0, 132003: 64.0, 132004: 25.0, 132005: 50.0, 132006: 30.0, 132007: 21.0, 132008: 10.0, 132009: np.nan, 132010: 30.0}}
    print(df)
    print(pd.DataFrame(correct))"""



if __name__ == "__main__":
    # files = glob.glob("/home/laiho/Documents/demos/mygames/*")#[30:31]
    files = glob.glob("/home/laiho/Documents/demos/bench_pro_demos/*")

    before = time.time()
    with mp.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files), desc="Parsing demos"))
    
    df = pd.concat(results)
    print(df)
    print(time.time() - before)
    #df = df[(df["distance"] > 30) & (df["noscope"] == True)]
    #print(df)


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