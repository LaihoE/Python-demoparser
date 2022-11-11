from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


# How many ticks after round started
afk_offset_ticks = 320

parser = DemoParser("demo.dem")
df = parser.parse_ticks(["moved_since_spawn"])
events = pd.DataFrame(parser.parse_events("round_freeze_end"))
events["tick"] += afk_offset_ticks
df = df[df["tick"].isin(events["tick"])]
print(df)
