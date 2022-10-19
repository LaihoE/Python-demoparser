from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def util_dmg(file):
    print(file)
    parser = DemoParser(file)
    game_events = parser.parse_events("round_mvp")
    print(pd.DataFrame(game_events))
    exit()
    #df = pd.DataFrame(game_events)
    #df = df[df["weapon"] == "hegrenade"]
    # Add file name to df
    #df["file"] = file
    #return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = pool.map(util_dmg, files)
    df = pd.concat(results)
    print(df)