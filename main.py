from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time
import tqdm
from collections import Counter
import csv
import joblib


"""def parse(file):
    number_of_kills = 5
    parser = DemoParser(file)
    before = time.time()
    #print(parser.parse_players())
    events = parser.parse_events("")
    df = parser.parse_props(["X", "Y", "Z", "velocity_X", "viewangle_yaw", "viewangle_pitch"])
    return file, pd.util.hash_pandas_object(df).sum()


if __name__ == "__main__":
    import tqdm
    files = glob.glob("/home/laiho/Documents/demos/dptest/*")[:50]
    #files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))
    d = dict(results)
    joblib.dump(d, "correct.pkl")"""

df = pd.read_csv("foo.csv", names=["key", "val"], sep="@")
print(df)
print(len(set(df["key"])))
print(df[df["key"] == 2622689])

"""import csv
with open("ok.csv", "a", newline="\n") as f:
    writer = csv.writer(f)
    for i in range(len(df)):
        v = df.iloc[i]["val"]
        k = df.iloc[i]["key"]
        l = v.split(",")
        if len(l) <= 1:
            vv = v.split("{")[-1]
            vv = vv.split("}")[0]
            for x in v:
                writer.writerow([k, vv])"""

df = pd.read_csv("ok.csv", names=["key", "val"])
print(df)

vals = []
keys = df["key"].unique()
for key in keys:
    subdf = df[df["key"] == key]
    uvs = set(subdf["val"])
    if len(uvs) > 1:
        print(key,uvs)
