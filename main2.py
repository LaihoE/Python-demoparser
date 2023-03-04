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
    #parser = DemoParser("/home/laiho/Documents/programming/rust/newparser/Python-demoparser/tests/test_demo.dem")
    parser = DemoParser(file)
    df = parser.parse_ticks(["ping"], ticks=[x for x in range(100000, 100001)])
    print(df)
    return df



if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")[35:36]
    #files = glob.glob("/home/laiho/Documents/demos/bench_pro_demos/*")

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