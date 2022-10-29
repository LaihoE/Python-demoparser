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
import joblib
import pandas as pd
from pandas.testing import assert_frame_equal


class TestFullDemo(unittest.TestCase):
    def setUp(self) -> None:
        demo_path = os.path.join(os.path.dirname(__file__), 'test.dem')
        self.parser = DemoParser(demo_path)

    def test_header(self):
        header = self.parser.parse_header()
        correct_header = joblib.load(os.path.join(os.path.dirname(
            __file__), 'correct_outputs', 'header.pkl'))
        self.assertEqual(header, correct_header)

    def test_events(self):
        events = self.parser.parse_events("")
        correct_path = os.path.join(os.path.dirname(
            __file__), 'correct_outputs', 'events.gz')
        with gzip.open(correct_path, 'rt', encoding='UTF-8') as zipfile:
            data = json.load(zipfile)
        self.assertEqual(events, data)

    """def test_players(self):
        players = pd.DataFrame(self.parser.parse_players())
        correct_players = joblib.load(os.path.join(os.path.dirname(
            __file__), 'correct_outputs', 'players.pkl'))
        players = players.reindex(
            sorted(players.columns), axis=1)
        self.assertEqual(assert_frame_equal(players, correct_players), True)"""

    def test_ticks(self):
        df = self.parser.parse_ticks(["X", "Y", "Z", "m_bIsScoped", "velocity_X",
                                      "velocity_Y", "velocity_Z",
                                      "viewangle_yaw", "viewangle_pitch",
                                      "health", "in_buy_zone",  "flash_duration"
                                      ])
        df = df.drop("name", axis=1)
        df = df.drop("steamid", axis=1)
        s = int(np.nansum(df.to_numpy()))
        self.assertEqual(s, 14427302575)
