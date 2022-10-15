from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm


def parse(file):
    number_of_kills = 5
    parser = DemoParser(file)
    before = time.time()
    #print(parser.parse_players())
    events = parser.parse_events("")
    df = parser.parse_props(["X"])#ticks=[x for x in range(10000, 10020)])
    print(df)


if __name__ == "__main__":
    import tqdm
    #files = glob.glob("/home/laiho/Documents/demos/faceits/test/*")
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))