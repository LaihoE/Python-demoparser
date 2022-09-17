from asyncio import events
from typing import List
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


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
    def __init__(self, file: str) -> None:
        self.path = file

    def parse_props(self, props) -> pd.DataFrame:
        out_arr = np.zeros((10000000), order='F')
        dims = demoparser.parse_props(self.path, prop_names, out_arr)
        df = transform_props(dims, out_arr, cols=prop_names)
        return df

    def parse_events(self, game_events) -> list:
        game_events = demoparser.parse_events(self.path, game_events)
        game_events = clean_events(game_events)
        return game_events


demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"

import glob
import time


prop_names = [
"m_vecVelocity[0]",
"m_vecVelocity[1]",
"m_vecVelocity[2]",
"m_vecViewOffset[2]",
"m_angEyeAngles[0]",
"m_angEyeAngles[1]",
"m_vecOrigin_X"
"m_vecOrigin_Y"
"m_vecOrigin[2]"]

event_name = "round_end"


parser = PythonDemoParser(demo_name)

game_events = parser.parse_events(event_name)
df = parser.parse_props(prop_names)
