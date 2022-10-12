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
    #df = parser.parse_props(["m_vecVelocity[0]"] )#ticks=[x for x in range(10000, 10020)])
    #parser.parse_header()
    events = parser.parse_events("player_death")
    #print(df)
    #print(df["m_vecVelocity[0]"].isna().sum())
    # print(time.time() - before)


if __name__ == "__main__":
    import tqdm
    files = glob.glob("/home/laiho/Documents/demos/faceits/test/*")
    #files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))