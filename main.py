from demoparser import DemoParser
import pandas as pd
import time

parser = DemoParser(
    "/home/laiho/Documents/demos/mygames/dd.dem")
"""print(pd.DataFrame(parser.parse_events("player_connect")))
print(pd.DataFrame(parser.parse_players()))

"""
df = pd.DataFrame(parser.parse_events(
    "player_jump", props=["X"]))
players = parser.parse_players()

#df = df[df["player_steamid"] == 76561198048924300]
print(df.columns)
print(df.loc[:, ["tick", "player_X", "player_steamid"]])


"""for p in players:
    if p["steamid"] == 76561198048924300:
        print(p)

before = time.time()
parser.parse_events("player_death")
print(time.time() - before)"""

# 100  46909      219.924316  76561198048924300

"""
185  148097  1021.277466  76561197992897289
186  148253   602.211731  76561198134270402
187  148361   646.589722  76561198258044111
188  148623  1172.984863  76561197999069229
189  148737   -78.504364  76561198194694750



Delta found at tick: 117966 start: 117990 val: F32(677.16846) ent:6
Delta found at tick: 118044 start: 118044 val: F32(405.40253) ent:9
Delta found at tick: 118996 start: 118996 val: F32(1040.8673) ent:5
Delta found at tick: 119844 start: 119844 val: F32(666.1038) ent:7
Delta found at tick: 121850 start: 121850 val: F32(476.41525) ent:3
Delta found at tick: 122076 start: 122076 val: F32(124.56452) ent:12
Delta found at tick: 125327 start: 126067 val: F32(223.96875) ent:10
Delta found at tick: 127371 start: 127371 val: F32(458.12128) ent:11
Delta found at tick: 127891 start: 127903 val: F32(126.56157) ent:6
Delta found at tick: 128069 start: 128069 val: F32(14.263841) ent:2
Delta found at tick: 128281 start: 128281 val: F32(396.55685) ent:9
Delta found at tick: 128939 start: 128939 val: F32(651.64496) ent:12
Delta found at tick: 130511 start: 130511 val: F32(-86.65525) ent:3
Delta found at tick: 130877 start: 130877 val: F32(-27.045773) ent:7
Delta found at tick: 133479 start: 133479 val: F32(650.3293) ent:10
Delta found at tick: 134581 start: 134581 val: F32(661.7746) ent:9
Delta found at tick: 134683 start: 134683 val: F32(1164.1848) ent:2
Delta found at tick: 134757 start: 134757 val: F32(1173.6451) ent:7
Delta found at tick: 135305 start: 135323 val: F32(1140.7435) ent:6
Delta found at tick: 135365 start: 135365 val: F32(656.17816) ent:12
Delta found at tick: 138725 start: 138725 val: F32(1112.4366) ent:12
Delta found at tick: 139089 start: 139089 val: F32(440.78122) ent:6
Delta found at tick: 139113 start: 139113 val: F32(863.54095) ent:2
Delta found at tick: 139135 start: 139135 val: F32(950.5324) ent:7
Delta found at tick: 139231 start: 139231 val: F32(722.83405) ent:10
Delta found at tick: 142075 start: 142075 val: F32(-140.64252) ent:10
Delta found at tick: 142631 start: 142631 val: F32(-124.79848) ent:6
Delta found at tick: 142497 start: 142841 val: F32(860.9933) ent:5
Delta found at tick: 142865 start: 142865 val: F32(244.59663) ent:12
Delta found at tick: 144359 start: 144359 val: F32(75.04462) ent:2
Delta found at tick: 145407 start: 145407 val: F32(311.25122) ent:7
Delta found at tick: 147961 start: 147961 val: F32(384.11884) ent:10
Delta found at tick: 148093 start: 148093 val: F32(780.28564) ent:2
Delta found at tick: 148077 start: 148097 val: F32(359.55063) ent:11
Delta found at tick: 148253 start: 148253 val: F32(378.15485) ent:6
Delta found at tick: 148361 start: 148361 val: F32(293.92288) ent:7
Delta found at tick: 148621 start: 148623 val: F32(804.8111) ent:3
Delta found at tick: 148737 start: 148737 val: F32(348.2599) ent:12






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


"""
18    3553  2656.021973  76561198048924300
21    4021  2631.000244  76561198048924300
24    4411  2678.225586  76561198048924300
26    4799   602.892700  76561198048924300
27    5085   677.680542  76561198048924300
40    6605  2643.068115  76561198048924300
43    9198 -1329.155762  76561198048924300
56   16680  -830.396057  76561198048924300
64   21562 -1550.193604  76561198048924300
71   26438  -786.321838  76561198048924300
75   29430  -520.667236  76561198048924300
82   34081 -1491.861450  76561198048924300
91   38735  -719.968750  76561198048924300
93   41911 -1863.942017  76561198048924300
100  46909    64.733200  76561198048924300
"""
