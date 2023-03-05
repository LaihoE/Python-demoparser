from demoparser import DemoParser
import pandas as pd
import glob
import multiprocessing as mp
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt
import numpy as np
import itertools
from pandas.testing import assert_frame_equal

def coordinates(file):
    if file == "/home/laiho/Documents/demos/faceits/cu/003309131115255562271_1824323488 (1).dem":
        return

    #print(file)
    parser = DemoParser(file)
    #df = parser.parse_ticks(["X"], ticks=[x for x in range(10000, 10001)])
    df = parser.parse_events("bomb_planted")
    # df = parser.parse_events("player_death")
    # print(df)
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[:1]

    with mp.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files), desc="Parsing demos"))
    
    df = pd.concat(results)
    print(df)
