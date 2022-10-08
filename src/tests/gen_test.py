from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm
import json
import gzip


parser = DemoParser("test.dem")
events = parser.parse_events("player_death")

with gzip.open("correct_outputs/player_deaths.gz", 'wt', encoding='UTF-8') as zipfile:
    json.dump(events, zipfile)
