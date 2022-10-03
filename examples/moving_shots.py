from asyncio import events
from typing import List
from unicodedata import name
import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from pyparser import PythonDemoParser
import multiprocessing
from sqlalchemy import create_engine




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


okfiles = okfiles[:400]

def first_bloods(file):
    
    parser = PythonDemoParser(file)
    game_events = parser.get_events("")
    return game_events
    


if __name__ == "__main__":
    import sqlite3
    import tqdm
    from collections import Counter
    before = time.time()
    conn = sqlite3.connect('all_events3')
    c = conn.cursor()
    with multiprocessing.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(first_bloods, okfiles), total=len(okfiles)))
    #df = pd.concat(results)
    engine = create_engine('sqlite:///all_events3', echo=False)

    sorted_game_events = {}

    for result in results:

        for event in result:
            if event["event_name"] in sorted_game_events:
                sorted_game_events[event["event_name"]].append(event)
            else:
                sorted_game_events[event["event_name"]] = [event]

    for event_name, event_list in sorted_game_events.items():
        df = pd.DataFrame(event_list) 
        df.to_sql(event_name, engine, if_exists='append')