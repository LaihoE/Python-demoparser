from demoparser import DemoParser
import pandas as pd
import time

parser = DemoParser(
    "/home/laiho/Documents/demos/mygames/1.dem")

df = pd.DataFrame(parser.parse_events_fast("player_death", props=["X"]))

print(df.columns)
print(df.loc[:, ["tick", "attacker_X", "player_steamid"]])


"""for p in players:
    if p["steamid"] == 76561198048924300:
        print(p)

before = time.time()
parser.parse_events("player_death")
print(time.time() - before)"""

# 100  46909      219.924316  76561198048924300
print(parser.parse_ticks(["X", "Y", "Z"]))
"""
Delta found at tick: 133913 start: 133913 val: F32(108.27322) ent:5
Delta found at tick: 135725 start: 135727 val: F32(-531.2773) ent:8
Delta found at tick: 136180 start: 138512 val: F32(1296.0) ent:10
Delta found at tick: 139079 start: 139079 val: F32(387.8189) ent:11
Delta found at tick: 139283 start: 139283 val: F32(155.59035) ent:8
Delta found at tick: 139767 start: 139767 val: F32(-1719.9042) ent:5
Delta found at tick: 141675 start: 141675 val: F32(-1066.8125) ent:2
Delta found at tick: 141775 start: 141775 val: F32(-684.79724) ent:9
Delta found at tick: 142165 start: 142165 val: F32(-359.3629) ent:3
Delta found at tick: 142637 start: 142637 val: F32(-126.58538) ent:4
Delta found at tick: 142745 start: 142777 val: F32(-680.03125) ent:7
Delta found at tick: 144877 start: 144877 val: F32(-95.969505) ent:3
Delta found at tick: 144919 start: 144919 val: F32(111.33612) ent:8
Delta found at tick: 145199 start: 145199 val: F32(60.360386) ent:10
Delta found at tick: 146219 start: 146219 val: F32(399.16318) ent:9
Delta found at tick: 147079 start: 147079 val: F32(133.50241) ent:7
Delta found at tick: 147969 start: 147969 val: F32(-1163.5692) ent:2
Delta found at tick: 148357 start: 148357 val: F32(-719.96875) ent:6
Delta found at tick: 148749 start: 148749 val: F32(135.86215) ent:11
Delta found at tick: 150886 start: 150886 val: F32(-167.79878) ent:7
Delta found at tick: 151066 start: 151076 val: F32(-777.46857) ent:2
Delta found at tick: 151898 start: 151898 val: F32(-2253.8918) ent:5
Delta found at tick: 151996 start: 151996 val: F32(-592.1735) ent:4
Delta found at tick: 152248 start: 152264 val: F32(-2162.2932) ent:3







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
