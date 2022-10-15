from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time


def parse(file):
    number_of_kills = 5
    parser = DemoParser("/home/laiho/Documents/programming/rust/search/t/9075236_9020546.dem")
    before = time.time()
    #print(parser.parse_players())
    events = parser.parse_events("")
    #df = parser.parse_props(["m_vecVelocity[0]"] )#ticks=[x for x in range(10000, 10020)])
    #print(df)
    exit()
    #parser.parse_header()
    #events = parser.parse_events("player_death")
    #print(df)
    #print(df["m_vecVelocity[0]"].isna().sum())
    # print(time.time() - before)


if __name__ == "__main__":
    import tqdm
    #files = glob.glob("/home/laiho/Documents/demos/faceits/test/*")
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))