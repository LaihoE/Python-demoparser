from glob import glob
import unittest
import numpy as np
import os
import glob
from demo_parser import PythonDemoParser
import pickle
import json




files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
parser = PythonDemoParser(files[0])


events = parser.parse_events("player_hurt")

with open('correct_outputs/events.json', 'w') as fp:
    json.dump(events, fp)