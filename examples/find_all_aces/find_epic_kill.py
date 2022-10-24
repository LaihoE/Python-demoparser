from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


number_of_kills = 5
parser = DemoParser("demo.dem")

df = pd.DataFrame(parser.parse_events("player_death"))
# end of "parser" after this its just pandas operations.

# Remove warmup rounds. Seems like something odd going on with round "1" so drop
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
    # Put all of those rounds in an output list
    all_rounds.append(kills_per_round)

print(all_rounds)