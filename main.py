import demoparser
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd



demo_name = "/home/laiho/Documents/demos/rclonetest/r.dem"
prop_name = [
"m_angEyeAngles[0]",
"m_angEyeAngles[1]",
"m_bSpotted"
]


z = np.zeros((1000000), order='F')
x = demoparser.parse_props(demo_name, prop_name)
print(x)