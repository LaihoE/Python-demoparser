from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import sqlite3


def parse(file):
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events("player_death"))
    return df


if __name__ == "__main__":
    files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")

    with mp.Pool(processes=12) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))
    df = pd.concat(results)

    # Creates db if not exists
    con = sqlite3.connect("testing_database.sqlite")
    df.to_sql("player_death", con=con, if_exists='append')