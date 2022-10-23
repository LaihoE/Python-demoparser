from ast import JoinedStr
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm
import random
import os


def coordinates(file):
    parser = DemoParser(file)
    max_round = min(int(parser.parse_events("player_death")[-1]["round"]), 30)

    df = parser.parse_ticks(["m_iMatchStats_Damage_Total", "m_iKills", "m_iAssists", "m_iDeaths"])
    damage = df.groupby(['steamid'], sort=False)['m_iMatchStats_Damage_Total'].max().reset_index()
    kills = df.groupby(['steamid'], sort=False)['m_iKills'].max().reset_index()
    assists = df.groupby(['steamid'], sort=False)['m_iAssists'].max().reset_index()
    deaths = df.groupby(['steamid'], sort=False)['m_iDeaths'].max().reset_index()

    joined = damage.merge(kills)
    joined = joined.merge(assists)
    joined = joined.merge(deaths)

    df = df.dropna()

    df["m_iKills"].astype("int")
    df["m_iAssists"].astype("int")
    df["m_iDeaths"].astype("int")

    joined["total_rounds"] = max_round
    joined["file"] = os.path.basename(file)
    #print(joined)
    #df.columns = ["steamid", "dmg_total", "kills"]
    return joined


if __name__ == "__main__":
    import time
    files = glob.glob("/home/laiho/Documents/demos/mygames/*")#[:30]
    print(files)
    before = time.time()
    with mp.Pool(processes=24) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(coordinates, files), total=len(files)))
    df = pd.concat(results)
    df = df.dropna()

    df.to_csv("data.csv")
    print(df)

    df = pd.concat(results)
    df = df.groupby(['steamid'], sort=False)['dmg'].mean().reset_index()
    df = df.sort_values("dmg")

    a = {}

    """"
    make, 76561198048924300
    Benu, 76561198134270402
    Emil, 76561198194694750
    Tohelo, 76561198036180618
    Lari, 76561198193238934
    Juuso, 76561198258044111
    Capu, 76561198066395116
    Oskari, 76561198073049527
    """

    a["make"] =  df[df["steamid"] == 76561198048924300]["dmg"].values[0]
    a["Benu"] = df[df["steamid"] == 76561198134270402]["dmg"].values[0]
    a["Emil"] =  df[df["steamid"] == 76561198194694750]["dmg"].values[0]
    a["Tohelo"] = df[df["steamid"] == 76561198036180618]["dmg"].values[0]
    a["Lari"] = df[df["steamid"] == 76561198193238934]["dmg"].values[0]
    a["Juuso"] = df[df["steamid"] == 76561198258044111]["dmg"].values[0]
    a["Capu"] = df[df["steamid"] == 76561198066395116]["dmg"].values[0]
    a["Oskari"] = df[df["steamid"] == 76561198073049527]["dmg"].values[0]
    
    """print("Name", "\t", "Average util dmg / game")
    for k,v in a.items():
        print(k,"\t", round(v, 2))"""