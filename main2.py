from demoparser import DemoParser
import glob
import time
from collections import Counter



files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
# files = glob.glob("/mnt/c/Users/emill/got/x/*")


okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)

path = okfiles[0]
path = "/home/laiho/Downloads/a/Elisa-Invitational-Fall-2022-eyeballers-vs-eclot-bo3/d.dem"
parser = DemoParser(path)


wanted_players = [76561197991348083]
wanted_ticks = [x for x in range(10000, 101000)]
df = parser.parse_props(["m_vecOrigin_X","m_iHealth" ,"m_vecOrigin_Y"], wanted_ticks, wanted_players)

print(df)