from asyncio import events
from typing import List
import demoparser
from pyparser import *
from pyparser import PythonDemoParser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import glob
import time








demo_path = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
parser = PythonDemoParser(demo_path)
death_info = []
death_events = parser.parse_events("player_death")
for death_event in death_events:
    tick = death_event["tick"]
    attacker = death_event["attacker"]
    print(tick, attacker)


