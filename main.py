from typing import List
import demoparser
from numpy import zeros
from pandas import DataFrame
import demoparser
import glob


def transform_props(dims, arr, cols):
    cols.append("tick")
    cols.append("entid")
    arr = arr[:dims[0]]
    arr = arr.reshape(dims[1], dims[2], order='F')
    d = {}
    k = ""
    v = ""
    for i in range(3, len(dims)):
        if i % 2 == 0:
            k = dims[i]
        else:
            v = dims[i]
            d[k] = v
    df = DataFrame(arr, columns=cols)
    df = df.replace({"entid": d})
    df["entid"].astype("int64")
    df["tick"].astype("int64")
    return df

def clean_events(events):
    cleaned_events = []
    for i in range(len(events)):
        subd = {}
        for k,v in events[i].items():
            subd[k] = v[0]
        cleaned_events.append(subd)
    return cleaned_events

class PythonDemoParser:
    def __init__(self, file: str) -> None:
        self.path = file

    def get_props(self, props_names, ticks=[], players=[]) -> DataFrame:
        out_arr = zeros((10_000_000), order='F')
        dims = demoparser.parse_props(self.path, props_names, out_arr, ticks, players)
        df = transform_props(dims, out_arr, cols=props_names)
        return df

    def get_events(self, game_events) -> list[dict]:
        game_events = demoparser.parse_events(self.path, game_events)
        game_events = clean_events(game_events)
        return [dict(sorted(game_event.items())) for game_event in game_events]
    
    def get_players(self) -> list[dict]:
        players = demoparser.parse_players(files[0])
        return [dict(sorted(player.items())) for player in players]
    
    def get_header(self) -> list[dict]:
        demo_header = demoparser.parse_header(files[0])
        return demo_header


# players = [76561198194694750]
files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
parser = PythonDemoParser(files[0])
df = parser.get_props(["m_hActiveWeapon", "m_iClip1"])
df = df[df["m_hActiveWeapon"] != -1]
weapons = set(df["m_hActiveWeapon"].to_list())
df = df[df["m_iClip1"] != -1]

# print(len(parser.get_events("weapon_fire")))

# & 0x7FF& 0x7FF
# for x in weapons:
    #print(int(x) & 0x7FF)