from asyncio import events
from typing import List
from unittest import result
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import multiprocessing

def transform_props(dims, arr, cols):
    cols.append("tick")
    arr = arr[:dims[0]]
    arr = arr.reshape(dims[1], dims[2], order='F')
    return pd.DataFrame(arr, columns=cols)

def clean_events(events):
    cleaned_events = []
    for i in range(len(events)):
        subd = {}
        for k,v in events[i].items():
            subd[k] = v[0]
        cleaned_events.append(subd)
    return cleaned_events

class PythonDemoParser:
    def __init__(self) -> None:
        pass

    def parse_props(self, props) -> pd.DataFrame:
        out_arr = np.zeros((10000000), order='F')
        dims = demoparser.parse_props(self.path, prop_names, out_arr)
        df = transform_props(dims, out_arr, cols=prop_names)
        return df

    def parse_events(self, game_events, path) -> list:
        game_events = demoparser.parse_events(path, game_events)
        game_events = clean_events(game_events)
        return pd.DataFrame(game_events)

    def parallel_events(self, game_events, files: List[str]):
        jobs = [(game_events, file) for file in files]
        with multiprocessing.Pool(processes=24) as pool:
            results = pool.starmap(self.parse_events, jobs)
        return pd.concat(results)


# demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
# demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

import glob
import time
import multiprocessing as mp
from collections import Counter
import uuid


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/mm/*")
    okfiles = []
    for file in files:
        if "info" not in file:
            okfiles.append(file)
    d = PythonDemoParser()
    df = d.parallel_events("bomb_abortplant", okfiles[:200])
    print(df)
    print(Counter(df["player_id"]))