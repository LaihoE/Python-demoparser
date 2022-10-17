from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def util_dmg(file):
    parser = DemoParser(file)
    game_events = parser.parse_events("player_hurt")
    df = pd.DataFrame(game_events)
    df = df[df["weapon"] == "hegrenade"]
    # Add file name to df
    df["file"] = file
    return df


if __name__ == "__main__":
    files = glob.glob("path/to/files/*")
    with mp.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(util_dmg, files), total=len(files)))
    df = pd.concat(results)
    print(df)