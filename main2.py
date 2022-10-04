import demoparser
import glob

files = glob.glob("/mnt/c/Users/emill/got/x/*")
okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)

path = okfiles[0]
props_names = ["m_vecVelocity[0]"]


df = demoparser.parse_props(path, props_names, [], [])
print(df)