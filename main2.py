from demoparser import DemoParser
import pandas as pd
import glob
import multiprocessing as mp
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt


# pd.set_option('display.max_rows', 500000)


def coordinates(file):

    parser = DemoParser(file)
    df2 = pd.DataFrame(parser.parse_ticks(["manager@m_iKills"], ticks=[x for x in range(40000, 40001)]))


if __name__ == "__main__":
    import numpy as np
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[:1]
    # files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[5]
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[15:16]
    # print(files)
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")#[80:81]
    # files = glob.glob("/home/laiho/Documents/demos/ow/*")

    with mp.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    a = np.concatenate(results)


"""
0  76561198055893769  40000               8.0
1  76561198134270402  40000               1.0
2  76561198189245325  40000               7.0
3  76561198211930059  40000               1.0
4  76561198272632133  40000               3.0
5  76561198362125234  40000               3.0
6  76561198980562947  40000               1.0
7  76561199005454535  40000               2.0
8  76561199029155762  40000               3.0
9  76561199066169302  40000               6.0
"""