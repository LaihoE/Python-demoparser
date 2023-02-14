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

    def test_player_ping(self):
        df = self.parser.parse_ticks(["manager@m_iPing"], ticks=[x for x in range(10000, 10001)])
        d = {'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'tick': {0: 10000, 1: 10000, 2: 10000, 3: 10000, 4: 10000, 5: 10000, 6: 10000, 7: 10000, 8: 10000, 9: 10000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'manager@m_iPing': {0: 31.0, 1: 29.0, 2: 30.0, 3: 18.0, 4: 24.0, 5: 15.0, 6: 11.0, 7: 5.0, 8: 35.0, 9: 43.0}}
        assert_frame_equal(pd.DataFrame(d), df, check_dtype=False)
    
    def test_player_health(self):
        df = self.parser.parse_ticks(["player@DT_BasePlayer.m_iHealth"], ticks=[x for x in range(30000, 30001)])
        d = {'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'player@DT_BasePlayer.m_iHealth': {0: 100.0, 1: 0.0, 2: 0.0, 3: 73.0, 4: 0.0, 5: 43.0, 6: 100.0, 7: 100.0, 8: 0.0, 9: 100.0}}
        assert_frame_equal(pd.DataFrame(d), df, check_dtype=False)

    def test_player_XY(self):
        df = self.parser.parse_ticks(["player@m_vecOrigin_X", "player@m_vecOrigin_Y"], ticks=[x for x in range(30000, 30001)])
        d = {'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'player@m_vecOrigin_X': {0: -557.94921875, 1: -1176.1212158203125, 2: -1733.4951171875, 3: -2286.39404296875, 4: -1176.1212158203125, 5: -1733.4951171875, 6: -1175.6988525390625, 7: -1945.1878662109375, 8: -1733.4951171875, 9: 1282.06103515625}, 'player@m_vecOrigin_Y': {0: -595.3869018554688, 1: -864.8699340820312, 2: -209.7762451171875, 3: -507.9263610839844, 4: -864.8699340820312, 5: -209.7762451171875, 6: -863.0912475585938, 7: -611.3868408203125, 8: -209.7762451171875, 9: -730.112548828125}}
        assert_frame_equal(pd.DataFrame(d), df, check_dtype=False)

    def test_player_death_event(self):
        df = self.parser.parse_events("player_death")
        df = df.iloc[20:30, :]
        d = {'assistedflash': {20: False, 21: False, 22: False, 23: False, 24: False, 25: False, 26: False, 27: False, 28: False, 29: False}, 'assister': {20: 0, 21: 0, 22: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 28: 0, 29: 0}, 'attacker': {20: 76561198147100782, 21: 76561198147100782, 22: 76561198189734257, 23: 76561198147100782, 24: 76561198201296319, 25: 76561198271657717, 26: 76561198089780719, 27: 76561198147100782, 28: 76561198189734257, 29: 76561198258044111}, 'attackerblind': {20: False, 21: False, 22: False, 23: False, 24: False, 25: False, 26: False, 27: False, 28: False, 29: False}, 'byte': {20: 2090798, 21: 2165886, 22: 2265720, 23: 2300514, 24: 2331310, 25: 2374610, 26: 2402974, 27: 2469120, 28: 2484344, 29: 2522920}, 'distance': {20: 20.318511962890625, 21: 22.209125518798828, 22: 17.52931022644043, 23: 23.070348739624023, 24: 23.784738540649414, 25: 22.05186653137207, 26: 1.8695234060287476, 27: 20.603708267211914, 28: 17.68633270263672, 29: 22.892305374145508}, 'dominated': {20: 0, 21: 0, 22: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 28: 0, 29: 0}, 'headshot': {20: True, 21: True, 22: True, 23: False, 24: False, 25: True, 26: True, 27: True, 28: True, 29: True}, 'noreplay': {20: False, 21: False, 22: False, 23: False, 24: False, 25: False, 26: False, 27: False, 28: False, 29: False}, 'noscope': {20: False, 21: False, 22: False, 23: False, 24: False, 25: False, 26: False, 27: False, 28: False, 29: False}, 'penetrated': {20: 0, 21: 0, 22: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 28: 0, 29: 0}, 'revenge': {20: 0, 21: 0, 22: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 28: 0, 29: 0}, 'steamid': {20: 76561198271657717, 21: 76561197993611582, 22: 76561198271657717, 23: 76561197993611582, 24: 76561198258044111, 25: 76561198147100782, 26: 76561198194694750, 27: 76561198271657717, 28: 76561197993611582, 29: 76561198201296319}, 'thrusmoke': {20: False, 21: False, 22: False, 23: False, 24: False, 25: False, 26: False, 27: False, 28: False, 29: False}, 'tick': {20: 5092, 21: 5284, 22: 5552, 23: 5618, 24: 5684, 25: 5790, 26: 5848, 27: 5986, 28: 6020, 29: 6102}, 'weapon': {20: 'glock', 21: 'usp_silencer', 22: 'famas', 23: 'ak47', 24: 'galilar', 25: 'ak47', 26: 'glock', 27: 'famas', 28: 'famas', 29: 'usp_silencer'}, 'weapon_fauxitemid': {20: '17293822569102704644', 21: '17293822569145761853', 22: '17293822569105784842', 23: '17293822569102704647', 24: '17293822569102704653', 25: '17293822569173942279', 26: '17293822569102704644', 27: '17293822569134948362', 28: '17293822569105784842', 29: '17293822569102704701'}, 'weapon_itemid': {20: '0', 21: '13487218802', 22: '16422319525', 23: '0', 24: '0', 25: '24516801954', 26: '0', 27: '16366965722', 28: '16422319525', 29: '0'}, 'weapon_originalowner_xuid': {20: '76561198147100782', 21: '76561198147100782', 22: '76561198189734257', 23: '76561198147100782', 24: '76561198201296319', 25: '76561198271657717', 26: '76561198089780719', 27: '76561198147100782', 28: '76561198189734257', 29: '76561198258044111'}, 'wipe': {20: 0, 21: 0, 22: 0, 23: 0, 24: 0, 25: 0, 26: 0, 27: 0, 28: 0, 29: 0}}
        assert_frame_equal(pd.DataFrame(d), df, check_dtype=False)

    def test_get_players(self):
        df = self.parser.parse_players()
        d = {'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'starting_team': {0: 'CT', 1: 'T', 2: 'CT', 3: 'T', 4: 'T', 5: 'CT', 6: 'T', 7: 'T', 8: 'CT', 9: 'CT'}}
        assert_frame_equal(pd.DataFrame(d), df, check_dtype=False)

    def test_header(self):
        header = self.parser.parse_header()
        h = {'network_protocol': '13819', 'protocol': '4', 'server_name': 'Valve CS:GO EU North Server (srcds8047-sto1.188.33)', 'playback_frames': '37039', 'signon_length': '567626', 'protoplayback_tickscol': '74195', 'client_name': 'GOTV Demo', 'playback_time': '1159.2969', 'map_name': 'de_mirage', 'game_dir': 'csgo'}
        self.assertDictEqual(header, h)