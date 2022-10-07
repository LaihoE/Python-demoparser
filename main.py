from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import time


"""parser = DemoParser("/home/laiho/Documents/demos/faceits/clean_unzompr/1.dem")
before = time.time()
#events = parser.parse_events("player_death")
players = parser.parse_players()
for player in players:
    print(sorted(player.items(), key=lambda x: x))
print(time.time() - before)

df = pd.DataFrame(parser.parse_events("player_death"))
players = parser.parse_players()
print(set(df["player_name"]))"""


def parse(file):
    number_of_kills = 5
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events("player_death"))
    players = parser.parse_players()
    print(set(df["player_name"]))
    for player in players:
        print(sorted(player.items(), key=lambda x: x))


if __name__ == "__main__":
    import tqdm

    files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    #files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    #files = files[:1]

    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))
