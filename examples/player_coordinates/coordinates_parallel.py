from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    wanted_props = ["X", "Y", "Z"]
    # This will early exit parsing after just 10k ticks
    wanted_ticks = [x for x in range(5000, 10000)]
    parser = DemoParser(file)
    # You can remove optional arguments to get all tick or all players
    df = parser.parse_ticks(wanted_props, ticks=wanted_ticks)
    return df


if __name__ == "__main__":
    files = glob.glob("/path/to/directory/with/demos/*")
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    df = pd.concat(results)
    print(df)
