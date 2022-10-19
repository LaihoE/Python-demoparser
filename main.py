from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    wanted_props = ["X", "Y", "Z"]
    wanted_ticks = [x for x in range(5000, 5050)]
    #print(file)
    parser = DemoParser(file)
    players = pd.DataFrame(parser.parse_events("player_death"))
    #print(players)
    #df = parser.parse_ticks(wanted_props, ticks=wanted_ticks)
    #print(players)
    #print(players)
    #players["entity_id"] = players["entity_id"].astype("int")
    #print(players.dtypes)
    #if players["entity_id"].max() > 11:
        #print(players)
    
    # You can remove optional arguments to get all tick or all players
    #
    #print(df)


if __name__ == "__main__":
    import time

    #files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    print(files)
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))
    print(time.time() - before)

    #df = pd.concat(results)
    #print(df)