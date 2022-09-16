from calendar import prcal
import numpy as np
import pandas as pd

#df = pd.read_html("events.html")
#print(df)

classes = []

with open("hurtevents.csv", "r") as f:
    rows = f.readlines()
    print(rows)
    for row in rows:
        