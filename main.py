from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time


def parse(file):
    number_of_kills = 5
    parser = DemoParser(file)
    df = parser.parse_props(["m_angEyeAngles[0]", "m_angEyeAngles[1]"], [], [])
    print(df)

if __name__ == "__main__":
    import tqdm
    files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))