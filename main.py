import new
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd



demo_name = "/home/laiho/Documents/demos/rclonetest/r.dem"
prop_name = [
"m_vecOrigin_X",
"m_vecOrigin_Y"
]




z = np.zeros((1000000), order='F')
x = new.parse(demo_name, prop_name, z)
z = z[:183446//2*len(prop_name)]
z = z.reshape(-1, len(prop_name), order='F')

df = pd.DataFrame(z, columns=prop_name)

#df = df[df["m_vecOrigin_X"] < 2000]
#df = df[df["m_vecOrigin_Y"] > -5000]

plt.scatter(df["m_vecOrigin_X"], df["m_vecOrigin_Y"])
plt.show()
"""
props = ["Coordinates", "angles"]
hurtevents = demo.get_game_events("hurtevent")

wanted_ticks = []
for hurtevent in hurtevents:
    wanted_ticks.append(hurtevent.tick)

df = demo.peek_props_at_ticks(props, wanted_indexes)
"""