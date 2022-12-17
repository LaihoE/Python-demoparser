import tqdm
import multiprocessing as mp
from demoparser import DemoParser
import pandas as pd
import time
import glob
from pandas.testing import assert_frame_equal


def coordinates(file):
    wanted_props = ["m_angEyeAngles[0]"]
    # This will early exit parsing after just 10k ticks
    parser = DemoParser(file)
    # You can remove optional arguments to get all tick or all players
    df = parser.parse_ticks(wanted_props, ticks=[x for x in range(1000, 10000)])
    #df = df.dropna()
    #df = df[df["steamid"] != 0]
    #print(df["steamid"].unique())
    print(df)



if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    df = pd.concat(results)
    print(df)
