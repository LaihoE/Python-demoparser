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



#demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
demo_name = "/home/laiho/Documents/demos/rclonetest/5.dem"
event_name = "announce_phase_end"

"""events = clean_events(demoparser.parse_events(demo_name, event_name))
print(events)
wanted_ticks = []"""

#for event in events:
    #print(event)
    #if event["userid"] == "3":
        #wanted_ticks.append(int(event["tick"]))
#print(wanted_ticks)
#exit()
import time

out_arr = np.zeros((10000000), order='F')
prop_names = ["m_vecVelocity[0]", "m_vecVelocity[1]", "m_vecVelocity[2]"]
before = time.time()
dims = demoparser.parse_props(demo_name, prop_names, out_arr)
print(time.time() - before)

"""
df = transform_props(dims, out_arr, cols=prop_names)
df = df[df["tick"].isin(wanted_ticks)]
print(df.iloc[:50, :])

"""

# df = df[df["m_vecOrigin_X"] < 2000]
# df = df[df["m_vecOrigin_Y"] > -6000] 
# killcoords = df[df["tick"].isin(wanted_ticks)]
# plt.scatter(killcoords["m_vecOrigin_X"], killcoords["m_vecOrigin_Y"])
# plt.show()