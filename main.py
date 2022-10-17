import chunk
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm
from collections import Counter
import csv


def parse(file):
    print(file)
    #before = time.time()
    parser = DemoParser(file)
    evs = parser.parse_events("player_death")
    #print(evs)
    #print(time.time() - before)
    #df = parser.parse_props(["X"], ticks=[x for x in range(10000)])
    #print(df.isna().sum())
    #print(evs)
    return evs


if __name__ == "__main__":
    import random

    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    #files = glob.glob("/media/laiho/New Volume/b/b/*")

    #x = random.shuffle(files)
    with mp.Pool(processes=24) as pool:
        results = pool.map(parse, files)
        print(results)