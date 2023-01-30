from demoparser import DemoParser
import pandas as pd

from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
from collections import Counter
import time
import matplotlib.pyplot as plt

def coordinates(file):
    #print(file)
    # parser = DemoParser("/home/laiho/Documents/demos/mygames/match730_003449965367076585902_0881240613_184.dem")
    parser = DemoParser(file)
    before = time.time()
    df = pd.DataFrame(parser.parse_ticks(["m_angEyeAngles[1]"]))
    #print(time.time() - before)
    print(df)



if __name__ == "__main__":
    import numpy as np
    # files = glob.glob("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/*")#[:1]
    # files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")#[2:3]
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            coordinates, files), total=len(files)))
    a = np.concatenate(results)

