from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd


def parse(file, n):
    parser = DemoParser(file)
    df = pd.DataFrame(parser.parse_events("player_death"))
    # Remove warmup rounds
    df = df[player_df["round"] != "0"]
    players = df["attacker_id"].unique()
    all_rounds = [] 

    for player in players:
        # slice players
        player_df = df[df["attacker_id"] == player]
        # Create df like this: ["round", "attacker_name", "total_kills"]
        kills_per_round = player_df.groupby(["round", "attacker_name", "attacker_id"]).size().to_frame(name = 'total_kills').reset_index()
        # Get rounds with kills > n
        kills_per_round = kills_per_round[kills_per_round["total_kills"] >= n]
        kills_per_round = kills_per_round[kills_per_round["attacker_id"].str.len()> 10]
        # Add file name to df
        kills_per_round["file"] = file
        # Put all of those rounds in an output list
        all_rounds.append(kills_per_round)
        
    return pd.concat(all_rounds)


if __name__ == "__main__":
    files = glob.glob("path/to/demos/*")
    files = [file for file in files if "info" not in file]

    number_of_kills = 5
    jobs = [(file, number_of_kills) for file in files]

    with mp.Pool(processes=12) as pool:
        results = pool.starmap(parse, jobs)

    df = pd.concat(results)
    print(df)