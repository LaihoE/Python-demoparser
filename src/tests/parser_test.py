from glob import glob
import json
import unittest
import numpy as np
import os
import glob
from demoparser import DemoParser


class TestFullDemo(unittest.TestCase):
    def setUp(self) -> None:
        self.files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
        self.parser = DemoParser(self.files[0])
    
    def test_events(self):
        events = self.parser.parse_events("player_hurt")
        correct_path = os.path.join(os.path.dirname(__file__), 'correct_outputs', 'events.json')
        with open(correct_path) as f:
            correct = json.load(f)
        self.assertEqual(events, correct)
    
    def test_no_duplicate_players(self):
        players = self.parser.parse_players()
        