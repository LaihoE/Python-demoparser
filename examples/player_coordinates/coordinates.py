from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def util_dmg(file):
    #print(file)
    wanted_props = ["m_vecOrigin_X", "m_angEyeAngles[0]"]
    wanted_players = [765195849165354]
    wanted_ticks = [x for x in range(100000)]

    parser = DemoParser(file)
    df = parser.parse_props(wanted_props,
                            ticks=wanted_ticks,
                            players=wanted_players)
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(util_dmg, files), total=len(files)))
    df = pd.concat(results)
    print(df)