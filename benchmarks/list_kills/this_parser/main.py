from demoparser import DemoParser
from glob import glob
from tqdm import tqdm


files = glob("/home/laiho/Documents/demos/bench_pro_demos/*")
for file in tqdm(files):
    parser = DemoParser(file)
    df = parser.parse_events("player_death")
    print(df.loc[:, ["attacker", "steamid"]])