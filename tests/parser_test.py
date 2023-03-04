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

    """def test_player_ping(self):
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
        self.assertDictEqual(header, h)"""

    """def test_weapons(self):
        df = self.parser.parse_ticks(["weapon", "ammo"], ticks=[x for x in range(10000, 25000)])
        df = df[df["tick"] == 22000]
        correct = {'tick': {132000: 22000, 132001: 22000, 132002: 22000, 132003: 22000, 132004: 22000, 132005: 22000, 132006: 22000, 132007: 22000, 132008: 22000, 132009: 22000, 132010: 22000}, 'name': {132000: 'Bert', 132001: 'mormoncrew', 132002: 'NN', 132003: 'KosatkaPeek mosambitchpeek', 132004: 'Bewer', 132005: 'Ganterhatced', 132006: 'Road to LE', 132007: 'duck', 132008: 'Psychopath', 132009: '-ExΩtiC-', 132010: 'krYtep'}, 'steamid': {132000: 0, 132001: 76561197993611582, 132002: 76561198089780719, 132003: 76561198134270402, 132004: 76561198147100782, 132005: 76561198189734257, 132006: 76561198194694750, 132007: 76561198201296319, 132008: 76561198229793868, 132009: 76561198258044111, 132010: 76561198271657717}, 'weapon': {132000: 'BaseWeaponWorldModel', 132001: 'molotov', 132002: 'awp', 132003: 'bizon', 132004: 'm4a1_silencer', 132005: 'p90', 132006: 'ak47', 132007: 'm4a4', 132008: 'awp', 132009: None, 132010: 'ak47'}, 'ammo': {132000: np.nan, 132001: -1.0, 132002: 10.0, 132003: 64.0, 132004: 25.0, 132005: 50.0, 132006: 30.0, 132007: 21.0, 132008: 10.0, 132009: np.nan, 132010: 30.0}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)


    def test_XY(self):
        df = self.parser.parse_ticks(["X", "Y"], ticks=[x for x in range(10000, 25000)])
        df = df[df["tick"] == 22000]
        correct = {'tick': {132000: 22000, 132001: 22000, 132002: 22000, 132003: 22000, 132004: 22000, 132005: 22000, 132006: 22000, 132007: 22000, 132008: 22000, 132009: 22000, 132010: 22000}, 'name': {132000: 'Bert', 132001: 'mormoncrew', 132002: 'NN', 132003: 'KosatkaPeek mosambitchpeek', 132004: 'Bewer', 132005: 'Ganterhatced', 132006: 'Road to LE', 132007: 'duck', 132008: 'Psychopath', 132009: '-ExΩtiC-', 132010: 'krYtep'}, 'steamid': {132000: 0, 132001: 76561197993611582, 132002: 76561198089780719, 132003: 76561198134270402, 132004: 76561198147100782, 132005: 76561198189734257, 132006: 76561198194694750, 132007: 76561198201296319, 132008: 76561198229793868, 132009: 76561198258044111, 132010: 76561198271657717}, 'X': {132000: -560.0, 132001: 526.1273193359375, 132002: -1181.72900390625, 132003: 56.75040054321289, 132004: -751.0429077148438, 132005: -2347.137451171875, 132006: -636.0609741210938, 132007: -1029.7215576171875, 132008: -520.0070190429688, 132009: -636.0609741210938, 132010: 100.56692504882812}, 'Y': {132000: -7680.0, 132001: -1624.7550048828125, 132002: -733.2579345703125, 132003: 807.7340087890625, 132004: 50.30073165893555, 132005: 412.8190002441406, 132006: -2295.5322265625, 132007: -2531.05615234375, 132008: -1053.8978271484375, 132009: -2295.5322265625, 132010: -1438.1640625}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)"""

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
        correct = {'tick': {0: 30000, 1: 30000, 2: 30000, 3: 30000, 4: 30000, 5: 30000, 6: 30000, 7: 30000, 8: 30000, 9: 30000}, 'name': {0: 'mormoncrew', 1: 'NN', 2: 'KosatkaPeek mosambitchpeek', 3: 'Bewer', 4: 'Ganterhatced', 5: 'Road to LE', 6: 'duck', 7: 'Psychopath', 8: '-ExΩtiC-', 9: 'krYtep'}, 'steamid': {0: 76561197993611582, 1: 76561198089780719, 2: 76561198134270402, 3: 76561198147100782, 4: 76561198189734257, 5: 76561198194694750, 6: 76561198201296319, 7: 76561198229793868, 8: 76561198258044111, 9: 76561198271657717}, 'm_iHealth': {0: 100, 1: 0, 2: 0, 3: 73, 4: 0, 5: 43, 6: 100, 7: 100, 8: 0, 9: 100}}
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

    def test_weapons(self):
        df = self.parser.parse_ticks(["weapon", "ammo"], ticks=[x for x in range(10000, 25000)])
        df = df[df["tick"] == 22000]
        correct = {'tick': {132000: 22000, 132001: 22000, 132002: 22000, 132003: 22000, 132004: 22000, 132005: 22000, 132006: 22000, 132007: 22000, 132008: 22000, 132009: 22000, 132010: 22000}, 'name': {132000: 'Bert', 132001: 'mormoncrew', 132002: 'NN', 132003: 'KosatkaPeek mosambitchpeek', 132004: 'Bewer', 132005: 'Ganterhatced', 132006: 'Road to LE', 132007: 'duck', 132008: 'Psychopath', 132009: '-ExΩtiC-', 132010: 'krYtep'}, 'steamid': {132000: 0, 132001: 76561197993611582, 132002: 76561198089780719, 132003: 76561198134270402, 132004: 76561198147100782, 132005: 76561198189734257, 132006: 76561198194694750, 132007: 76561198201296319, 132008: 76561198229793868, 132009: 76561198258044111, 132010: 76561198271657717}, 'weapon': {132000: 'BaseWeaponWorldModel', 132001: 'molotov', 132002: 'awp', 132003: 'bizon', 132004: 'm4a1_silencer', 132005: 'p90', 132006: 'ak47', 132007: 'm4a4', 132008: 'awp', 132009: None, 132010: 'ak47'}, 'ammo': {132000: np.nan, 132001: -1.0, 132002: 10.0, 132003: 64.0, 132004: 25.0, 132005: 50.0, 132006: 30.0, 132007: 21.0, 132008: 10.0, 132009: np.nan, 132010: 30.0}}
        assert_frame_equal(pd.DataFrame(correct), df, check_dtype=False)

