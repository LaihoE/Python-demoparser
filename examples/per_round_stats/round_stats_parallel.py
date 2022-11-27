from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def round_stats(file):

    wanted_player = 76561197991348083
    parser = DemoParser(file)

    df = parser.parse_ticks(["total_damage", "mvps", "round"])
    df = df[df["steamid"] == wanted_player]
    df = df.loc[:, ["total_damage", "mvps", "round"]]
    df = df.drop_duplicates()
    return df


if __name__ == "__main__":
    files = glob.glob("/path/to/dir/with/demos/*")

    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            round_stats, files), total=len(files)))
    df = pd.concat(results)
    
    print(df)