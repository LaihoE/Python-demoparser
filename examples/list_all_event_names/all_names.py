from demoparser import DemoParser

parser = DemoParser("demo.dem")
game_events = parser.parse_events("")
for event in game_events:
    print(event["event_name"])
