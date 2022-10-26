from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    print(file)
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events(
    "player_death", props=["X", "Y", "Z", "weapon_name"]))
    #print(df.iloc[:40, :])
    print(df.loc[:40, ["tick", "attacker_X", "attacker_Y", "player_X", "player_Y", "attacker_name", "player_name"]])
    #print(pd.DataFrame(parser.parse_players()))

if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/mnt/d/b/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))
    df = pd.concat(results)
