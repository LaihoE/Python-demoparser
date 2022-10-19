from demoparser import DemoParser
import pandas as pd


# Filters are commented out.

wanted_props = ["X", "Y", "Z"]
parser = DemoParser("/home/laiho/Documents/demos/faceits/cu/*")
 # We can also pass optional arguments players and ticks. That are list of integers.
df = parser.parse_ticks(wanted_props)


print(df)