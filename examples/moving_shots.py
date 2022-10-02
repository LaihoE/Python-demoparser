from asyncio import events
from typing import List
from unicodedata import name
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from pyparser import PythonDemoParser
import multiprocessing



demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
#demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

import glob
import time




# files = glob.glob("/home/laiho/Documents/demos/mm/*")
# files = glob.glob("/media/laiho/New Volume1/demos/testc/*")
files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")


okfiles = []
for x in files:
        if "info" not in x:
            okfiles.append(x)



def first_bloods(file):
    print("*******")
    parser = PythonDemoParser(file)
    df = pd.DataFrame(parser.get_events("player_hurt"))
    #df = df.groupby(["attacker_id"])["round"].max()
    #print(df)
    return df


if __name__ == "__main__":
    import sqlite3
    import tqdm
    from collections import Counter
    before = time.time()

    with multiprocessing.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(first_bloods, okfiles), total=len(okfiles)))

    df = pd.concat(results)
    
    conn = sqlite3.connect('test_database')
    
    df.to_sql('player_hurt', conn)
    print(time.time() - before)
    print(df)
    #df.to_csv("test.csv")
    # print(Counter(df["weapon"]))