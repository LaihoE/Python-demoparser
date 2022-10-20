from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random

def coordinates(file):
    wanted_props = ["X", "Y", "Z"]
    wanted_ticks = [x for x in range(5000, 5050)]
    #print(file)
    parser = DemoParser(file)
    df = parser.parse_ticks(wanted_props, players=[76561198194694750], ticks=wanted_ticks)
    print(df)
    """players = pd.DataFrame(parser.parse_players())
    print(players)
    if len(players) >= 5:
        rng = random.randint(0, 5)
        sid = int(players.iloc[rng]["steamid"])
        df = parser.parse_ticks(wanted_props, players=[76561198194694750], ticks=wanted_ticks)
        #print(df)"""



if __name__ == "__main__":
    import time

    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    #files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    #print(files)
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))
    print(time.time() - before)

    #df = pd.concat(results)
    #print(df)