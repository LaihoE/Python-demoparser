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
        parser = DemoParser("test.dem")
        df = parser.parse_ticks(["X","Y", "Z", "m_bIsScoped", "velocity_X",
                                "velocity_Y", "velocity_Z",
                                "viewangle_yaw", "viewangle_pitch",
                                "health", "in_buy_zone",  "flash_duration"
                                ])
        df = df.drop("name", axis=1)
        df = df.drop("steamid", axis=1)
        s = int(np.nansum(df.to_numpy()))
        self.assertEqual(s, 14428487448)