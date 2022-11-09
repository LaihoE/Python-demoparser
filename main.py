import tqdm
import multiprocessing as mp
from demoparser import DemoParser
import pandas as pd
import time
import glob
from pandas.testing import assert_frame_equal


before = time.time()
files = glob.glob("/home/laiho/Documents/demos/mygames/*")

"""
files = []
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003418549680613621938_0984855660_183.dem")
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003564210687548850263_1135999279_184.dem")
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003439547603925074007_0749396926_184.dem")
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003434548923417493974_0621905606_184.dem")
files.append("/home/laiho/Documents/demos/mygames/e.dem")

files.append(
    "/home/laiho/Documents/demos/mygames/match730_003421736756800651551_0387184790_184.dem")
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003425781074100224332_0845925734_132.dem")

files = []
# files.append("/home/laiho/Documents/demos/mygames/e.dem")
files.append(
    "/home/laiho/Documents/demos/mygames/match730_003439547603925074007_0749396926_184.dem")
"""
# files = ["/home/laiho/Documents/demos/mygames/match730_003564210687548850263_1135999279_184.dem"]
#files = ["/home/laiho/Documents/demos/mygames/match730_003575550120617312564_1824169885_187.dem"]
for file in files:
    parser = DemoParser(file)
    print(file)
    df = pd.DataFrame(parser.parse_events_fast(
        "player_death", props=["X", "Y", "Z"]))
    # print(df["m_vecOrigin_Xlol"])
    df = df.iloc[1:, :]

    """df = pd.DataFrame(parser.parse_events_fast(
        "player_death", props=["X", "Y", "Z"])).round(3)
    print(df.columns)
    # if "precuimplayer_X" in df.columns:
    #print(df["precuimplayer_X"], df["m_vecOrigin_Xlol"])
    print(pd.DataFrame(parser.parse_players()))
    print(pd.DataFrame(parser.parse_events("player_connect")))"""

    """dfm = pd.read_csv(
        "/home/laiho/Documents/programming/rust/demoparse/tests/gen_go_out/killevent.csv")
    dfm = dfm.loc[:, ["player_X", "player_Y", "player_Z",
                      "player_name", "player_steamid", "attacker_steamid"]]
    # print(df)
    for i in range(len(df)):
        print(df.iloc[i]["m_vecOrigin_Xlol"], df.iloc[i]["m_vecOrigin_Ylol"],
              dfm.iloc[i]["player_X"], dfm.iloc[i]["player_Y"])"""

    #dfm = dfm.iloc[40:, :]

    #a = a[a["tick"] > 10000]
    #b = b[b["tick"] > 10000]

    # for sid in dfm["player_steamid"].unique():
    # print("markus", sid, dfm[dfm["player_steamid"] == sid])

    """for i in range(len(df)):
        if df.iloc[i]["precuimplayer_X"] != df.iloc[i]["m_vecOrigin_Xlol"]:
            print(df.iloc[i]["precuimplayer_X"], df.iloc[i]
                  ["m_vecOrigin_Xlol"], df.iloc[i]["tick"], df.iloc[i]["player_name"], df.iloc[i]["player_steamid"])"""

    """for i in df:
        if len(i) != 32:
            print(len(i), i)
    """
    """df = pd.DataFrame(parser.parse_events_fast(
        "player_death", props=["X", "Y", "Z"])).round(3)

    df2 = pd.DataFrame(parser.parse_events(
        "player_death", props=["X", "Y", "Z"])).round(3)

    fast = df.loc[:, ["m_vecOrigin_X", "m_vecOrigin_Y",
                      "m_vecOrigin[2]", "player_name", "player_steamid", "attacker_steamid", "tick"]]

    fast.columns = ["player_X", "player_Y", "player_Z",
                    "player_name", "player_steamid", "attacker_steamid", "tick"]
    fast = fast.iloc[40:, :]
    df2 = df2.iloc[40:, :]

    a = df2.loc[:, ["player_X", "player_Y", "player_Z",
                    "player_name", "player_steamid", "attacker_steamid", "tick"]]
    b = fast.loc[:, ["player_X", "player_Y", "player_Z",
                     "player_name", "player_steamid", "attacker_steamid", "tick"]]

    a = a[a["player_steamid"] != 0]
    b = b[b["player_steamid"] != 0]

    a = a[a["attacker_steamid"] != 0]
    b = b[b["attacker_steamid"] != 0]

    for sid in b["player_steamid"].unique():
        print(b[b["player_steamid"] == sid])
    print("******************", type(sid))
    for sid in a["player_steamid"].unique():
        print(a[a["player_steamid"] == sid])
    
    print("******************", b["attacker_steamid"].dtype)
    print("NANNANNANNANNANNANNANNANNANNAN", b.iloc[11])

    print("******************", a["attacker_steamid"].dtype)
    print("NANNANNANNANNANNANNANNANNANNAN", a.iloc[11])

    dfm = pd.read_csv(
        "/home/laiho/Documents/programming/rust/demoparse/tests/gen_go_out/killevent.csv")
    dfm = dfm.loc[:, ["player_X", "player_Y", "player_Z",
                      "player_name", "player_steamid", "attacker_steamid"]]
    dfm = dfm.iloc[40:, :]

    #a = a[a["tick"] > 10000]
    #b = b[b["tick"] > 10000]

    # for sid in dfm["player_steamid"].unique():
    # print("markus", sid, dfm[dfm["player_steamid"] == sid])

    # print("\U0001F923")
    # print(pd.DataFrame(parser.parse_events("player_connect")))
    # print(pd.DataFrame(parser.parse_players()))
    assert_frame_equal(a, b)"""


"""
for i in range(len(fast)):
    print(fast.iloc[i].to_list())
print(pd.DataFrame(parser.parse_events("player_connect")))
"""
