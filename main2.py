from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    print(file)
    parser = DemoParser(file)
    #df = pd.DataFrame(parser.parse_players())
    df = parser.parse_ticks(["health", "m_vecOrigin_X", "Y", "ammo"])
    print(df)
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/mnt/d/b/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))
    df = pd.concat(results)
