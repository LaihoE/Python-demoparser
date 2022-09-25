from asyncio import events
from typing import List
from unicodedata import name
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def transform_props(dims, arr, cols):
    cols.append("tick")
    cols.append("entid")
    arr = arr[:dims[0]]
    arr = arr.reshape(dims[1], dims[2], order='F')
    d = {}
    k = ""
    v = ""
    for i in range(3, len(dims)):
        if i % 2 == 0:
            k = dims[i]
        else:
            v = dims[i]
            d[k] = v
    df = pd.DataFrame(arr, columns=cols)
    df = df.replace({"entid": d})
    df["entid"].astype("int64")
    df["tick"].astype("int64")
    return df

def clean_events(events):
    cleaned_events = []
    for i in range(len(events)):
        subd = {}
        for k,v in events[i].items():
            subd[k] = v[0]
        cleaned_events.append(subd)
    return cleaned_events

class PythonDemoParser:
    def __init__(self, file: str) -> None:
        self.path = file

    def parse_props(self, props_names) -> pd.DataFrame:
        out_arr = np.zeros((10000000), order='F')
        dims = demoparser.parse_props(self.path, props_names, out_arr)
        df = transform_props(dims, out_arr, cols=props_names)
        return df

    def parse_events(self, game_events) -> list:
        game_events = demoparser.parse_events(self.path, game_events)
        game_events = clean_events(game_events)
        return game_events


demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
#demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

import glob
import time



event_name = "round_stadftgsrt"
files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
deaths = []
rounds_ends = []

from collections import Counter

#file = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

import time

# BENU 76561198134270402
# EMIL 76561198194694750

total_attackers = []
total_victims = []

files = glob.glob("/home/laiho/Documents/demos/mm/*")
okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)

name_map = {
    "76561198134270402": "benu",
    "76561198194694750": "laiho",
    "76561198073049527": "osku",
    "76561198048924300": "make",
    "76561198258044111": "juuso",
    "76561198193238934": "lari",
}


ok = ["76561198134270402",
    "76561198194694750",
    "76561198073049527",
    "76561198048924300",
    "76561198258044111",
    "76561198193238934",]


def util_dmg(file):
    parser = PythonDemoParser(file)
    game_events = parser.parse_events("player_hurt")
    df = pd.DataFrame(game_events)
    return df
    

if __name__ == "__main__":
    import multiprocessing
    with multiprocessing.Pool(processes=24) as pool:
        results = pool.map(util_dmg, okfiles[:200])
    
    df = pd.concat(results)
    weapons = df["weapon"].unique()
    for weapon in weapons:
        print(weapon)
        filtered = df[df["weapon"] == weapon]
        filtered = filtered.astype({"dmg_health": int})
        res = filtered.groupby(["attacker_id"], as_index=False).sum().sort_values("dmg_health")
        res = res[res["attacker_id"].isin(ok)]
        res = res.replace({"attacker_id": name_map})
        plt.title(weapon)
        plt.bar(res["attacker_id"], res["dmg_health"])
        plt.savefig(f"all_weapons_last_200/{weapon}.png")
        plt.show()