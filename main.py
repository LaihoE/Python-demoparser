from demoparser import DemoParser
import pandas as pd


parser = DemoParser("/home/laiho/Documents/demos/mygames/ww.dem")
df = pd.DataFrame(parser.parse_events(
    "player_death", props=["viewangle_yaw"]))
players = parser.parse_players()

df = df[df["player_steamid"] == 76561198048924300]
print(df.columns)
print(df.loc[:, ["tick", "player_viewangle_yaw", "player_steamid"]])


for p in players:
    if p["steamid"] == 76561198048924300:
        print(p)


# 100  46909      219.924316  76561198048924300

"""
18    3553            358.698120  76561198048924300
21    4021            337.626343  76561198048924300
24    4411             11.601562  76561198048924300
26    4799            358.341064  76561198048924300
27    5085              3.999023  76561198048924300
40    6605             14.924927  76561198048924300
43    9198            155.813599  76561198048924300
56   16680            269.555054  76561198048924300
64   21562            297.053833  76561198048924300
71   26438            280.354614  76561198048924300
75   29430            161.636353  76561198048924300
82   34081            138.746338  76561198048924300
91   38735            281.315918  76561198048924300
93   41911             56.425781  76561198048924300
100  46909            219.924316  76561198048924300
"""
