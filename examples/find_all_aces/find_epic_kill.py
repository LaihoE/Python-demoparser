from demoparser import DemoParser
import glob
import multiprocessing as mp
import pandas as pd
import tqdm


number_of_kills = 5

parser = DemoParser("demo.dem")
df = pd.DataFrame(parser.parse_events("player_death", rounds=True))
# end of "parser" after this its just pandas operations.

all_rounds = []
# Remove warmup rounds
df = df[df["round"] != 0]

players = df["attacker_steamid"].unique()
for player in players:
    # slice players
    player_df = df[df["attacker_steamid"] == player]
    # Create df like this: ["round", "attacker_name", "total_kills"]
    kills_per_round = player_df.groupby(["round", "attacker_name", "attacker_steamid"]).size(
    ).to_frame(name='total_kills').reset_index()
    # Get rounds with kills > n
    kills_per_round = kills_per_round[kills_per_round["total_kills"]
                                      >= number_of_kills]
    # Add file name to df
    # Put all of those rounds in an output list
    all_rounds.append(kills_per_round)

print(pd.concat(all_rounds))
