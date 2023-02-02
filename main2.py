from demoparser import DemoParser
import pandas as pd
import glob
import multiprocessing as mp
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt


# pd.set_option('display.max_rows', 500000)


def coordinates(file):
    print(file)
    # parser = DemoParser("/home/laiho/Documents/demos/mygames/match730_003449965367076585902_0881240613_184.dem")
    parser = DemoParser(file)
    before = time.time()
    df = pd.DataFrame(parser.parse_ticks(["DT_CSPlayer.m_angEyeAngles[0]"], ticks=[x for x in range(40000, 40001)]))
    #print(df["column_1"].unique())
    print(df)



if __name__ == "__main__":
    import numpy as np
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[:1]
    # files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")#[5]
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")#[15:16]
    print(files)
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    a = np.concatenate(results)


    """
    76561198029122943
    90 10000
    76561198048924300
    90 10000
    76561198055893769
    90 10000
    76561198061642773
    90 10000
    76561198112665670
    90 10000
    76561198122925075
    90 10000
    76561198134270402
    0 10000
    76561198189245325
    0 10000
    76561198845955287
    90 10000
    76561198029122943
    6.8884277 10000
    76561198048924300
    0.115356445 10000
    76561198055893769
    357.40173 10000
    76561198061642773
    0.1977539 10000
    76561198112665670
    7.7783203 10000
    76561198122925075
    1.5380859 10000
    76561198134270402
    4.6307373 10000
    76561198189245325
    7.706909 10000
    76561198829733633
    17.122192 10000
    76561198845955287
    4.3066406 10000



    CORRECT
    76561198029122943
    40 10000
    76561198048924300
    90 10000
    76561198055893769
    90 10000
    76561198061642773
    90 10000
    76561198112665670
    90 10000
    76561198122925075
    90 10000
    76561198134270402
    90 10000
    76561198189245325
    0 10000
    76561198829733633
    90 10000
    76561198845955287
    90 10000
    76561198029122943
    6.8884277 10000
    76561198048924300
    0.5932617 10000
    76561198055893769
    2.4829102 10000
    76561198061642773
    0.1977539 10000
    76561198112665670
    7.7783203 10000
    76561198122925075
    1.5380859 10000
    76561198134270402
    0.05493164 10000
    76561198189245325
    7.706909 10000
    76561198829733633
    17.122192 10000
    76561198845955287
    4.3066406 10000

    """