from typing import List
import demoparser
from numpy import zeros
from pandas import DataFrame
import demoparser
import glob
import polars as pl

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
    df = DataFrame(arr, columns=cols)
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
    return [dict(sorted(game_event.items())) for game_event in cleaned_events]

class PythonDemoParser:
    def __init__(self, file: str) -> None:
        self.path = file

    def get_props(self, props_names: list[str], ticks=[], players=[]) -> DataFrame:
        if type(props_names) != list:
            raise TypeError("Wanted values should be a list not a string!")
        df = demoparser.parse_props(self.path, props_names, ticks, players)
        df = pl.DataFrame(df).to_pandas()
        props_names.extend(["tick", "steamid", "name"])
        df.columns = props_names
        return df

    def get_events(self, game_events) -> list[dict]:
        game_events = demoparser.parse_events(self.path, game_events)
        game_events = clean_events(game_events)
        return [dict(sorted(game_event.items())) for game_event in game_events]

    def get_players(self) -> list[dict]:
        players = demoparser.parse_players(self.path)
        return [dict(sorted(player.items())) for player in players]

    def get_header(self) -> list[dict]:
        demo_header = demoparser.parse_header(self.path)
        return demo_header

import time

# players = [76561198194694750]
# files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")

files = glob.glob("/mnt/c/Users/emill/got/x/*")
okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)


import struct

class Bytebuffer:
    def __init__(self, data):
        self.data = data
        self.index = 0

    def read(self, num_bytes):
        data = self.data[self.index:self.index + num_bytes]
        self.index += num_bytes
        return data

    def read_int(self):
        data = self.read(4)
        data = struct.unpack("<I", data)[0]
        return data

    def read_short(self):
        return struct.unpack("<H", self.read(2))[0]


for x in okfiles:
    print(x)
    parser = PythonDemoParser(x)
    df = parser.get_props(["m_vecVelocity[0]", "m_szLastPlaceName", "m_vecOrigin_X"], ticks=[x for x in range(10000)])
    print(df)


"""
m_vecOrigin
m_vecOrigin[2]
m_Local.m_aimPunchAngle
m_Local.m_aimPunchAngleVel
m_Local.m_flFOVRate
m_Local.m_flFallVelocity
m_Local.m_flLastDuckTime
m_Local.m_viewPunchAngle
m_vecVelocity[0]
"""