from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm


def parse(file):
    parser = DemoParser(file)
    before = time.time()
    evs = parser.parse_events("player_death")
    #df = parser.parse_props(["m_vecVelocity[0]"], ticks=[x for x in range(10000, 10020)])
    print(time.time() - before)

if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))