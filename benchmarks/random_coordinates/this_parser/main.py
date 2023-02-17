from demoparser import DemoParser
from glob import glob
from tqdm import tqdm


files = glob("/home/laiho/Documents/demos/bench_pro_demos/*")
wanted_ticks = [x*10000 for x in range(10)]

for file in tqdm(files):
    parser = DemoParser(file)
    df = parser.parse_ticks(["player@m_vecOrigin_X", "player@m_vecOrigin_Y"], ticks=wanted_ticks)
    print(df)