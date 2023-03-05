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
        demo_path = os.path.join(os.path.dirname(__file__), 'test_demo.dem')
        self.parser = DemoParser(demo_path)   

    def test_bomb_plant_coordinates(self):
        df = self.parser.parse_events("bomb_planted", props=["X", "Y"])
        correct = {'site': {0: 455, 1: 454, 2: 454, 3: 454}, 'tick': {0: 9563, 1: 15181, 2: 22693, 3: 30994}, 'user_X': {0: -1905.6671142578125, 1: -252.3131561279297, 2: -253.96875, 3: -296.5209045410156}, 'user_Y': {0: 242.03125, 1: -2139.345947265625, 2: -2142.978515625, 3: -2162.304931640625}, 'userid': {0: 76561198258044111, 1: 76561198271657717, 2: 76561197993611582, 3: 76561198271657717}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)

    def test_player_coordinates(self):
        df = self.parser.parse_ticks(["X", "Y"], ticks=[x for x in range(40000, 40001)])
        correct = {'tick': {0: 40000, 1: 40000, 2: 40000, 3: 40000, 4: 40000, 5: 40000, 6: 40000, 7: 40000, 8: 40000, 9: 40000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'X': {0: 399.9996032714844, 1: -1924.303955078125, 2: 224.3791961669922, 3: -1105.8050537109375, 4: -1912.6451416015625, 5: 532.0077514648438, 6: -286.2942199707031, 7: -728.06005859375, 8: 76.82417297363281, 9: 745.6975708007812}, 'Y': {0: 42.7064208984375, 1: -497.55181884765625, 2: 15.238286972045898, 3: -660.165771484375, 4: -405.3867492675781, 5: -1629.3917236328125, 6: -2101.557861328125, 7: -1841.8876953125, 8: 811.3328857421875, 9: -1188.4967041015625}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)

    def test_player_health(self):
        df = self.parser.parse_ticks(["health"], ticks=[x for x in range(30000, 30001)])
        correct = {'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'health': {0: 100, 1: 0, 2: 0, 3: 73, 4: 0, 5: 43, 6: 100, 7: 100, 8: 0, 9: 100}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)

    def test_header(self):
        header = self.parser.parse_header()
        h = {'network_protocol': '13819', 'protocol': '4', 'server_name': 'Valve CS:GO EU North Server (srcds8047-sto1.188.33)', 'playback_frames': '37039', 'signon_length': '567626', 'protoplayback_tickscol': '74195', 'client_name': 'GOTV Demo', 'playback_time': '1159.2969', 'map_name': 'de_mirage', 'game_dir': 'csgo'}
        self.assertDictEqual(header, h)

    def test_get_players(self):
        df = self.parser.parse_players()
        d = {'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'starting_team': {0: 'CT', 1: 'T', 2: 'CT', 3: 'T', 4: 'T', 5: 'CT', 6: 'T', 7: 'T', 8: 'CT', 9: 'CT'}}
        assert_frame_equal(pd.DataFrame(df), df, check_dtype=False)

    def test_round_win_event(self):
        df = self.parser.parse_events("round_end")
        correct = {'legacy': {0: 0, 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0, 7: 0, 8: 0, 9: 0, 10: 0, 11: 0, 12: 0, 13: 0}, 'message': {0: '#SFUI_Notice_Terrorists_Win', 1: '#SFUI_Notice_Terrorists_Win', 2: '#SFUI_Notice_Terrorists_Win', 3: '#SFUI_Notice_Bomb_Defused', 4: '#SFUI_Notice_Bomb_Defused', 5: '#SFUI_Notice_CTs_Win', 6: '#SFUI_Notice_CTs_Win', 7: '#SFUI_Notice_CTs_Win', 8: '#SFUI_Notice_CTs_Win', 9: '#SFUI_Notice_CTs_Win', 10: '#SFUI_Notice_CTs_Win', 11: '#SFUI_Notice_CTs_Win', 12: '#SFUI_Notice_CTs_Win', 13: '#SFUI_Notice_Terrorists_Surrender'}, 'nomusic': {0: 0, 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0, 7: 0, 8: 0, 9: 0, 10: 0, 11: 0, 12: 0, 13: 0}, 'player_count': {0: 10, 1: 10, 2: 10, 3: 10, 4: 10, 5: 10, 6: 10, 7: 10, 8: 10, 9: 10, 10: 9, 11: 9, 12: 9, 13: 9}, 'reason': {0: 9, 1: 9, 2: 9, 3: 7, 4: 7, 5: 8, 6: 8, 7: 8, 8: 8, 9: 8, 10: 8, 11: 8, 12: 8, 13: 17}, 'tick': {0: 11933, 1: 15387, 2: 18938, 3: 24715, 4: 31826, 5: 38120, 6: 45744, 7: 50504, 8: 53842, 9: 58685, 10: 64433, 11: 69143, 12: 72095, 13: 72594}, 'winner': {0: 2, 1: 2, 2: 2, 3: 3, 4: 3, 5: 3, 6: 3, 7: 3, 8: 3, 9: 3, 10: 3, 11: 3, 12: 3, 13: 3}}
        assert_frame_equal(pd.DataFrame(df), df, check_dtype=False)

    def test_kills(self):
        df = self.parser.parse_ticks(["m_iKills"], ticks=[x for x in range(30000, 30001)])
        correct = {'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'm_iKills': {0: 2, 1: 2, 2: 4, 3: 4, 4: 2, 5: 6, 6: 4, 7: 2, 8: 4, 9: 3}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)

    def test_ping(self):
        df = self.parser.parse_ticks(["ping"], ticks=[x for x in range(30000, 30001)])
        correct = {'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'ping': {0: 31, 1: 28, 2: 26, 3: 17, 4: 25, 5: 24, 6: 5, 7: 5, 8: 28, 9: 43}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)
