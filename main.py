from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time



def parse(file):
    print(file)
    number_of_kills = 5
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events("player_death"))
    players = parser.parse_players()
    #print(df.dtypes)
    #print(set(df["player_name"]))

if __name__ == "__main__":
    import tqdm

    files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")

    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))
