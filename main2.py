from demoparser import DemoParser
import pandas as pd
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import time
import os


def coordinates(file):
    print(file)
    parser = DemoParser(file)

    df = parser.parse_ticks(["total_damage", "round"])
    df = df[df["steamid"] == 76561198194694750]
    #df = df.loc[:, ["total_damage", "round"]]
    df = df.drop_duplicates()
    print(df)
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob(
        "/home/laiho/Documents/demos/mygames/*")#[:1]  # [:100]

    with mp.Pool(processes=1) as pool:
        results = list(pool.map(coordinates, files))
    df = pd.concat(results)
    print(df)