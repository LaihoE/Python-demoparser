from asyncio import events
from typing import List
from unicodedata import name
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from pyparser import PythonDemoParser




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
    game_events = parser.parse_events("player_death")
    df = pd.DataFrame(game_events)
    #df = df[df["weapon"] == "m4a1"]
    df = df[df["attacker_id"] == "76561198194694750"]
    df = df[df["round"] != "0"]
    try:
        # SELECT sum(*) group by round
        # x = df.groupby("round").size().values.max()
        print(len(df))
    except Exception as e:
        pass
    return df, file


if __name__ == "__main__":
    import multiprocessing
    print(len(okfiles))
    with multiprocessing.Pool(processes=2) as pool:
        results = pool.map(first_bloods, okfiles)
    print(results)
    #df = pd.concat(results)
    #print(df.groupby("round").size())