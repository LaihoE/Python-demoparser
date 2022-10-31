from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    # How many ticks after round started
    afk_offset_ticks = 320
    
    parser = DemoParser(file)
    df = parser.parse_ticks(["moved_since_spawn"])
    events = pd.DataFrame(parser.parse_events("round_freeze_end"))
    events["tick"] += afk_offset_ticks
    df = df[df["tick"].isin(events["tick"])]
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))

    df = pd.concat(results)
    df = df[df["moved_since_spawn"] == 0]
    print(Counter(df["steamid"]))