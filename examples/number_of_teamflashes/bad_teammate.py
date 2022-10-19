from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def util_dmg(file):
    parser = DemoParser(file)
    #players = parser.parse_players()
    #print(players)
    game_events = parser.parse_events("")

    for event in game_events:
        print(event["event_name"])
    exit()
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(util_dmg, files), total=len(files)))
    df = pd.concat(results)
    print(df)