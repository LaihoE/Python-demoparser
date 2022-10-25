from demoparser import DemoParser


parser = DemoParser("test.dem")
players = parser.parse_players()
for player in players:
    print(player["name"], player["steamid"], player["crosshair_code"])