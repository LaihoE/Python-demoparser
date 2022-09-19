from asyncio import events
from typing import List
import demoparser
from parser import *
from parser import PythonDemoParser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import glob
import time



demo_path = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
demo_path = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

parser = PythonDemoParser(demo_path)


tick_id_pairs = []
death_events = parser.parse_events("player_death")
# Create a list of (tick, attacker-id) for later parsing
for death_event in death_events:
    tick = death_event["tick"]
    attacker = death_event["attacker"]
    tick_id_pairs.append((tick, attacker))

# m_vecOrigin is the prop for coordinate of the player
out_arr = np.zeros(10000000, order="F")
df = parser.parse_props(["m_vecVelocity[0]", "m_vecVelocity[1]"])
print(df)
#print(df[df["m_vecOrigin_X"]!= -1] )
