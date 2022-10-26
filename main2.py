from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    print(file)
    parser = DemoParser(file)
    #df = parser.parse_header()
    df = parser.parse_ticks(["health"], ticks=[x for x in range(1000)])
    #df = pd.DataFrame(parser.parse_events("weapon_fire", props=["X", "Y", "Z"]))
    #df = df[df["player_steamid"] == 76561198362125234]
    #df = df.loc[:, ["attacker_m_vecOrigin_Y", "attacker_m_vecOrigin_X", "tick"]]
    print(df)
    return df


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/mnt/d/b/mygames/*")
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))
    df = pd.concat(results)
