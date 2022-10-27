from demoparser import DemoParser


wanted_props = ["X", "Y", "Z"]
parser = DemoParser("demo.dem")
# We can also pass optional arguments players and ticks.
df = parser.parse_ticks(wanted_props)

print(df)
