from demoparser import DemoParser
import pandas as pd
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import time


def coordinates(file):
    print(file)
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events_fast(
        "player_death", props=["X", "Y", "Z"]))
    # print(df["attacker_m_vecOrigin_X"])
    # print(df.columns)
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(pool.map(coordinates, files))
    df = pd.concat(results)
    print(df)
    print(time.time() - before)
