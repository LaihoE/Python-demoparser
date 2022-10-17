from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    wanted_props = ["X", "Y", "Z"]
    wanted_players = [765195849165354] 
    wanted_ticks = [x for x in range(100000)]

    parser = DemoParser(file)
    # You can remove optional arguments to get all tick or all players
    df = parser.parse_props(wanted_props,
                            ticks=wanted_ticks,
                            players=wanted_players)
    return df


if __name__ == "__main__":
    files = glob.glob("path/to/my/demos/*") # remember * at the end
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))
    df = pd.concat(results)
    print(df)