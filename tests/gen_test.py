from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm
import json
import gzip
import numpy as np
import collections


def gen_events_tests(file):
    parser = DemoParser(file)
    events = parser.parse_events("")
    with gzip.open("correct_outputs/events.gz", 'wt', encoding='UTF-8') as zipfile:
        json.dump(events, zipfile)

def gen_tick_tests(file):
    parser = DemoParser(file)
    df = parser.parse_ticks(["X", "Y", "Z", "m_bIsScoped", "velocity_X",
                            "velocity_Y", "velocity_Z",
                             "viewangle_yaw", "viewangle_pitch",
                             "health", "in_buy_zone",  "flash_duration"
                             ], players=[76561198194694750])
    df = df.drop("name", axis=1)
    df = df.drop("steamid", axis=1)
    s = int(np.nansum(df.to_numpy()))
    return file, s

def gen_header(file):
    parser = DemoParser(file)
    header = parser.parse_header()
    joblib.dump(header, "correct_outputs/header.pkl")

def gen_players(file):
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_players())    #.sort_values("comp_wins").set_index("steamid").to_numpy().flatten()
    df = df.reindex(sorted(df.columns), axis=1)
    print(df)
    joblib.dump(df, "correct_outputs/players.pkl")


if __name__ == "__main__":
    import random
    import joblib
    file = "test.dem"
    gen_tick_tests(file)
    gen_header(file)
    gen_events_tests(file)
    gen_players(file)
    