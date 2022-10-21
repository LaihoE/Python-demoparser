from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random


def coordinates(file):
    wanted_props = ["X"]
    wanted_ticks = [x for x in range(5000, 5050)]
    #print(file)
    parser = DemoParser(file)
    df = parser.parse_ticks(["m_iDeaths"])
    return set(df["m_iDeaths"])


if __name__ == "__main__":
    import time
    files = glob.glob("/mnt/d/Trash-1000/fa/3/*")#[:10]
    print(files)
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))
    all_w = []
    for x in results:
        for i in x:
            if str(i) != "nan":
                all_w.append(int(i))    

    s = set(all_w)
    print(max(s))
    print(min(s))


    print(time.time() - before)