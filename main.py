from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm


def parse(file):
    parser = DemoParser(file)
    df = parser.parse_props(["m_vecVelocity[0]"], ticks=[x for x in range(10000, 10020)])


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))