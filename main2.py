from ast import JoinedStr
from tkinter.tix import Tree
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os


def coordinates(file):

    before = time.time()
    parser = DemoParser(file)
    print(parser.parse_header())
    events = parser.parse_ticks(["X"], ticks=[153860])
    print(time.time() - before)
    # print(events)


if __name__ == "__main__":
    import time
    files = glob.glob("/home/laiho/Documents/demos/faceits/average/*")  # [:30]
    print(files)
    before = time.time()
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    df = pd.concat(results)
