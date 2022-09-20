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


#demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
#demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem"

import glob
import time
import multiprocessing as mp
from collections import Counter
import uuid


if __name__ == "__main__":
    prop_names = [
    "m_vecVelocity[0]",
    "m_vecVelocity[1]",
    ]

    event_name = "player_death"
    files = glob.glob("/home/laiho/Documents/demos/mm/*")
    #files.extend(glob.glob("/media/laiho/New Volume/5kcheaters/5/b/*"))
    hs = []

    def parse_file(fileq, dataq):
        while fileq.qsize() > 0:

            file = fileq.get()
            print(file)

            parser = PythonDemoParser(file)
            hurts = parser.parse_events(event_name)
            df = pd.DataFrame(hurts)
            df.to_csv(f"data/{uuid.uuid4()}.csv")
            #dataq.put(df)


        """df = pd.DataFrame(hs)
        df = df[df["weapon"] == "p90"]
        d = Counter(df["attacker"])
        df = pd.DataFrame.from_records(d.most_common(), columns=['name','count'])
        df = df[df["name"] != "23"]
        print(df.iloc[:50, :])"""




    import os
    old_files = glob.glob("data/*")
    for old_file in old_files:
        os.remove(old_file)

    fileq = mp.Queue()
    for file in files:
        if file[-1] != "o" and file != "/home/laiho/Documents/demos/mm/match730_003535786327695949955_1269260600_190.dem":
            if file != "/home/laiho/Documents/demos/mm/match730_003533265574882705732_0879717267_183.dem":
                if file != "/home/laiho/Documents/demos/mm/match730_003451971608577573241_0748266049_188.dem":
                    if file != "/home/laiho/Documents/demos/mm/match730_003431375114384441554_0028193074_190.dem":
                        if file != "/home/laiho/Documents/demos/mm/match730_003536129379618783361_1932237007_182.dem":
                            fileq.put(file)

    print(len(files))

    before = time.time()
    dataq = mp.Queue()
    processes = [mp.Process(target=parse_file, args=(fileq, dataq)) for x in range(24)]
    for p in processes:
        p.start()
    for p in processes:
        p.join()

    print(100 / (time.time() - before), "DEMOS PER SECOND", time.time() - before)



    df = pd.concat([pd.read_csv(f) for f in glob.glob("data/*")])
    df = df[df["weapon"] == "m4a1"]
    d = Counter(df["attacker"])
    df = pd.DataFrame.from_records(d.most_common(), columns=['name','count'])
    df = df[df["name"] != "23"]
    print(df.iloc[:50, :])