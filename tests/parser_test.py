from glob import glob
import json
import unittest
import numpy as np
import os
import glob
from demoparser import DemoParser
import gzip
import pickle
import multiprocessing as mp
import tqdm

class TestFullDemo(unittest.TestCase):
    def setUp(self) -> None:
        demo_path = os.path.join(os.path.dirname(__file__), 'test.dem')
        self.parser = DemoParser(demo_path)
    
    def test_events(self):
        events = self.parser.parse_events("")
        correct_path = os.path.join(os.path.dirname(__file__), 'correct_outputs', 'events.gz')
        with gzip.open(correct_path, 'rt', encoding='UTF-8') as zipfile:
            data = json.load(zipfile)
        self.assertEqual(events, data)

    def test_ticks(self):
        files = glob.glob("/home/laiho/Documents/demos/faceits/cu/*")[:10]
        with open("sums.pkl", 'rb') as handle:
            correct_sums = pickle.load(handle)
        with mp.Pool(processes=12) as pool:
            results = list(tqdm.tqdm(pool.imap_unordered(_paralell_parse, files), total=len(files)))
        for t in results:
            summed = t[0]
            file = t[1]
            self.assertEqual(summed - correct_sums[file], 0)
    
def _paralell_parse(file):
    parser = DemoParser(file)
    df = parser.parse_ticks(["X","Y", "Z", "m_bIsScoped", "velocity_X",
                            "velocity_Y", "velocity_Z",
                            "viewangle_yaw", "viewangle_pitch",
                            "health", "in_buy_zone",  "flash_duration"
                            ])
    df = df.drop("name", axis=1)
    df = df.drop("steamid", axis=1)
    summed = int(np.nansum(df.to_numpy()))
    return summed, file
