"""
0  -765.008362  76561198029122943     80000    41.677444  76561198029122943     80000
1  1447.925537  76561198048924300     80000   544.051758  76561198048924300     80000
2   820.091797  76561198055893769     80000   621.792847  76561198055893769     80000
3  -804.959595  76561198061642773     80000  -891.931824  76561198061642773     80000
4   229.163910  76561198112665670     80000  2135.673584  76561198112665670     80000
5   607.526733  76561198122925075     80000  1943.032104  76561198122925075     80000
6    19.723381  76561198134270402     80000   377.004150  76561198134270402     80000
7   513.004456  76561198189245325     80000  -132.960098  76561198189245325     80000
8  -765.008362  76561198829733633     80000    41.677444  76561198829733633     80000
9  -246.615295  76561198845955287     80000  1784.494751  76561198845955287     80000

df = pd.DataFrame(parser.parse_ticks(["X", "Y"], ticks=[x for x in range(80000, 80001)]))
"""


"""
0  76561198029122943  80000           -765.008362                                         1.0
1  76561198048924300  80000           1447.925537                                         3.0
2  76561198055893769  80000            820.091797                                         7.0
3  76561198061642773  80000           -804.959595                                         0.0
4  76561198112665670  80000            229.163910                                         0.0
5  76561198122925075  80000            607.526733                                         0.0
6  76561198134270402  80000             19.723381                                         1.0
7  76561198189245325  80000            513.004456                                         1.0
8  76561198829733633  80000           -765.008362                                         0.0
9  76561198845955287  80000           -246.615295                                         0.0
"""
from collections import Counter
v = []
with open("data.txt") as f:
    data = f.readlines()
    for x in data:
        v.append((x.split("[")[-1].split("]")[0]))

z = Counter(v).most_common()
z.reverse()


import pandas as pd

df = pd.DataFrame(z, columns=["key","n"])
df2 = df[df["n"] > 10000]

print(df2["n"].sum())
print(df["n"].sum())

s = 0
for (idx, x) in enumerate(z):
    s += x[-1]
    print(idx, x, s)