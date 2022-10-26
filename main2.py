from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    print(file)
    parser = DemoParser(file)
    #df = pd.DataFrame(parser.parse_players())
    df = parser.parse_ticks(["health", "m_vecOrigin_X", "Y", "weapon_name"])
    # df = pd.DataFrame(parser.parse_events(
    # "player_footstep", props=["X", "Y", "Z", "weapon_name"]))

    print(df)
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))
    df = pd.concat(results)
