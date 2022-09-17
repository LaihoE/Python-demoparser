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



demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"

import glob
import time
out_arr = np.zeros((10000000), order='F')
files = glob.glob("/home/laiho/Documents/demos/benchmark/*")


before = time.time()
for demo_name in files:

    # demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"

    prop_names = ["m_vecOrigin_X", "m_vecOrigin_Y"]

    dims = demoparser.parse_props(demo_name, prop_names, out_arr)

    df = transform_props(dims, out_arr, cols=prop_names)


print(time.time() - before)
# 10s