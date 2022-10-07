from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def util_dmg(file):

    wanted_props = ["m_vecOrigin_X", "m_vecOrigin_X"]
    wanted_players = [] # Empty for all players
    wanted_ticks = [] # =10000..11000

    parser = DemoParser(file)
    df = parser.parse_props(wanted_props,
                            wanted_ticks,
                            wanted_players)
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    with mp.Pool(processes=2) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(util_dmg, files), total=len(files)))
    df = pd.concat(results)
    print(df)