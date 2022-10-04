from demoparser import DemoParser
import glob
import time
from collections import Counter


files = glob.glob("/mnt/c/Users/emill/got/x/*")
okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)

path = okfiles[0]


parser = DemoParser(path)


df = parser.parse_props(["m_iHealth"], [], [])
print(df)