from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd


def parse(file):
    parser = DemoParser(file)
    df = parser.parse_events("player_death")
    return df

# files = glob.glob("/mnt/d/b/b/*")
files = glob.glob("/mnt/d/5kcheaters/5/a/*")

with mp.Pool(processes=12) as pool:
    results = pool.map(parse, files)

df = pd.concat(results)