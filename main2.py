from demoparser import DemoParser
import pandas as pd

from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def coordinates(file):
    print(file)
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_ticks(["X", "Y", "Z"]))
    print(df)
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=1) as pool:
        results = list(pool.map(coordinates, files))
    df = pd.concat(results)
    print(df)
