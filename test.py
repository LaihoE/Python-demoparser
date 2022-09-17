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



import glob
import time
out_arr = np.zeros((100000000), order='F')
demo_name = "/home/laiho/Documents/demos/benchmark/1.dem"


before = time.time()



dims = demoparser.parse_props(demo_name, prop_names, out_arr)
a = transform_props(dims, out_arr, cols=prop_names).to_numpy()

correct = np.load("1.dem.npy")
passed = int(correct.sum()) == int(a.sum())

print("PASSED:", passed)


# 10s