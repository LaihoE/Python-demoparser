from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time


def parse(file):
    number_of_kills = 5
    parser = DemoParser(file)
    before = time.time()
    #print(parser.parse_players())
    df = parser.parse_props(["m_bInBuyZone"], ticks=[x for x in range(10000)], players=[76561198066395116])
    # events = parser.parse_events("round_end")
    print(df)
    print(time.time() - before)


if __name__ == "__main__":
    import tqdm
    files = glob.glob("/home/laiho/Documents/demos/test/*")[:1]
    #files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))