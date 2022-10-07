from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


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


if __name__ == "__main__":

    files = glob.glob("/home/laiho/Documents/demos/mygames/*")
    #files = glob.glob("/home/laiho/Documents/demos/faceits/clean_unzompr/*")
    with mp.Pool(processes=8) as pool:
        results = list(tqdm.tqdm(pool.imap_unordered(parse, files), total=len(files)))

    df = pd.concat(results)
    print(df)