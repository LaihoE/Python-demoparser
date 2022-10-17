from demoparser import DemoParser
import pandas as pd


wanted_props = ["X", "Y", "Z"]
wanted_players = [765195849165354]
wanted_ticks = [x for x in range(100000)]

parser = DemoParser("demo.dem")
# You can remove optional arguments to get all tick or players
df = parser.parse_ticks(wanted_props,
                        ticks=wanted_ticks,
                        players=wanted_players)

print(df)