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




files = glob.glob("/home/laiho/Documents/demos/mm/*")
okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)


def first_bloods(file):
    parser = PythonDemoParser(file)
    df = pd.DataFrame(parser.get_events("hostage_hurt"))
    return df



if __name__ == "__main__":
    import tqdm
    from collections import Counter
    before = time.time()

    with multiprocessing.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(first_bloods, okfiles), total=len(okfiles)))
    df = pd.concat(results)
    print(time.time() - before)

    c = Counter(df["player_name"])
    plt.barh(c.keys(), c.values())
    plt.show()