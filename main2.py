import demoparser
import glob


files = glob.glob("/home/laiho/Documents/demos/rclonetest/*")
#files = glob.glob("/mnt/c/Users/emill/got/x/*")


okfiles = []
for file in files:
        if "info" not in file:
            okfiles.append(file)

path = okfiles[0]
print(path)
parser = demoparser.DemoParser(path)
print(parser.parse_events("player_hurt"))