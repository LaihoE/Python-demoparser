from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):

    wanted_player = 76561197991348083
    parser = DemoParser(file)

    df = parser.parse_ticks(["total_damage", "mvps", "round"])
    df = df[df["steamid"] == wanted_player]
    df = df.drop_duplicates()
    return df


if __name__ == "__main__":
    files = glob.glob(
        "/home/laiho/Documents/demos/mygames/*")

    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    df = pd.concat(results)
    
    print(df)