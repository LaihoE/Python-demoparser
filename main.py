from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm


def parse(file):
    parser = DemoParser(file)
    before = time.time()
    df = parser.parse_props(["m_hPlayerPing"], ticks=[x for x in range(20000, 200050)])
    print(df)
    print(time.time() - before)


if __name__ == "__main__":
    files = glob.glob("/mnt/d/test_demos/faceit/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))