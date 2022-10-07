from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd


def parse(file):

    number_of_kills = 5

    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events("player_death"))
    # Remove warmup rounds
    df = df[df["round"] != "0"]
    df = df[df["round"] != "1"]

    # Here we can include any other filters like weapons etc.
    # df = df[df["weapon"] == "ak47"]

    all_rounds = []
    players = df["attacker_id"].unique()
    for player in players:
        # slice players
        player_df = df[df["attacker_id"] == player]
        # Create df like this: ["round", "attacker_name", "total_kills"]
        kills_per_round = player_df.groupby(["round", "attacker_name", "attacker_id"]).size().to_frame(name = 'total_kills').reset_index()
        # Get rounds with kills > n
        kills_per_round = kills_per_round[kills_per_round["total_kills"] >= number_of_kills]
        # Make sure all ids are fine
        kills_per_round = kills_per_round[kills_per_round["attacker_id"].str.len()> 10]
        # Add file name to df
        kills_per_round["file"] = file
        # Put all of those rounds in an output list
        all_rounds.append(kills_per_round)
    return pd.concat(all_rounds)
from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd


def parse(file):
    number_of_kills = 5

    parser = DemoParser(file)
    wanted_props = ["m_vecOrigin_X", "m_iHealth"]
    wanted_players = [76561198977304502] # Empty for all players
    wanted_ticks = [x for x in range(10000, 15000)] # =10000..11000

    parser = DemoParser(file)
    df = parser.parse_props(wanted_props,
                            wanted_ticks,
                            wanted_players)
    #print(df[df["steamid"] == "76561198977304502"])
    print(df)
    #print(parser.parse_header())




if __name__ == "__main__":
    import tqdm

    files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    files = files[:1]

    with mp.Pool(processes=1) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))
