import pandas as pd
from collections import Counter
import numpy as np

import pandas as pd
  
# Creating the Series
sr = pd.Series([11, 21, 8, 18, 65, 18, 32, 10, 5, 32, None])
sr = np.log2(sr)
print(sr)



"""df.to_csv("comptest/commm3.txt", index=False)

df = df.sort_values("byte")
df.to_csv("comptest/commm2.txt", index=False)"""

"""df = df.loc[:, ["byte", "tick"]]
df = df.drop_duplicates()
df = df.sort_values("byte")

a = df["byte"].to_numpy().astype("uint32")
b = df["tick"].to_numpy().astype("uint32")
c = np.concatenate((a, b))


np.save("comptest/mapper", c)

print(df)"""

"""u = set()
print(len(df["byte"].unique()))

df = df.loc[:, ["byte", "tick", "pidx"]]
df = df.drop_duplicates()
print(df)

l = list(df.itertuples(index=False, name=None))

import numpy as np


d = {}
for idx, t in enumerate(l):
    d[t[0]] = idx

df = pd.read_csv("coma.txt", sep=" ", names=["byte", "tick", "entid", "pidx"])

df["idx"] = df["byte"].map(d)
print(df)
df = df.loc[:, ["entid", "idx", "pidx"]]
#df = df.sort_values("pidx")


a = df["entid"].to_numpy().astype("int32")
b = df["idx"].to_numpy().astype("int32")
c = df["pidx"].to_numpy().astype("int32")

print(df)
print(a.shape, b.shape)

d = np.concatenate((a, b, c))


np.save("comptest/a", a)
np.save("comptest/b", b)
np.save("comptest/c", d)

df.to_csv("comptest/out.csv", index=False)"""