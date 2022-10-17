from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm
import json
import gzip
import numpy as np

def gen_events_tests():

    parser = DemoParser("test.dem")
    events = parser.parse_events("")

    with gzip.open("correct_outputs/events.gz", 'wt', encoding='UTF-8') as zipfile:
        json.dump(events, zipfile)


def gen_tick_tests(file):
    parser = DemoParser(file)
    df = parser.parse_ticks(["X","Y", "Z", "m_bIsScoped", "velocity_X",
                            "velocity_Y", "velocity_Z",
                            "viewangle_yaw", "viewangle_pitch",
                            "health", "in_buy_zone",  "flash_duration"
                            ])
    df = df.drop("name", axis=1)
    df = df.drop("steamid", axis=1)
    s = int(np.nansum(df.to_numpy()))
    return file, s

if __name__ == "__main__":
    import random
    import joblib

    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")[:100]
    files = ["test.dem"]
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(gen_tick_tests, files), total=len(files)))
    print(results)
    #d = dict(results)
    #joblib.dump(d, "sums.pkl")