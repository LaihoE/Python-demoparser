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
        demo_path = os.path.join(os.path.dirname(__file__), '1.dem')
        self.parser = DemoParser(demo_path)

    def test_player_death_event(self):
        events = pd.DataFrame(self.parser.parse_events_fast("player_death", props=["X", "Y", "Z"])).round(3)

        correct_path = os.path.join(os.path.dirname(
            __file__), 'correct_outputs', 'killevent_markus_parser.csv')
        dfgo = pd.read_csv(correct_path)

        a = events.loc[:, ["player_X", "player_Y", "player_Z",
                           "attacker_X", "attacker_Y", "attacker_Z", "attacker_steamid"]]
        b = dfgo.loc[:, ["player_X", "player_Y", "player_Z",
                         "attacker_X", "attacker_Y", "attacker_Z", "attacker_steamid"]]

        self.assertEqual(a.equals(b), True)
