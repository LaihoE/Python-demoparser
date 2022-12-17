import pandas as pd


df = pd.read_csv("lol.txt", names=["bits", "idx"])

df["bits"] = df["bits"].astype(int)

s = ""
for i in range(2000):
    #print(i, df[df["idx"] == i]["bits"].unique())
    uniq = df[df["idx"] == i]["bits"].unique()
    if len(uniq) == 1:
        s += str(uniq[0]) + ","
    else:
        s += "0,"

print(s)