from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


def parse(file):
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_players())
    #df = parser.parse_ticks(["m_iTeamNum"])
    print(df)


if __name__ == "__main__":
    from collections import Counter
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")[:10]
    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(
            parse, files), total=len(files)))
