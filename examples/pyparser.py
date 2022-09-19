import pandas as pd
import numpy as np


def transform_props(dims, arr, cols):
    cols.append("tick")
    arr = arr[:dims[0]]
    arr = arr.reshape(dims[1], dims[2], order='F')
    return pd.DataFrame(arr, columns=cols)

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

    def parse_props(self, props) -> pd.DataFrame:
        out_arr = np.zeros((10000000), order='F')
        dims = demoparser.parse_props(self.path, props, out_arr)
        df = transform_props(dims, out_arr, cols=props)
        return df

    def parse_events(self, game_events) -> list:
        game_events = demoparser.parse_events(self.path, game_events)
        game_events = clean_events(game_events)
        return game_events