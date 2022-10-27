from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


parser = DemoParser("demo.dem")
game_events = parser.parse_events("player_hurt")
df = pd.DataFrame(game_events)
df = df[df["weapon"] == "hegrenade"]
print(df)
