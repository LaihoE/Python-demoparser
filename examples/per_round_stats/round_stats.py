from demoparser import DemoParser


wanted_steamid = 76561197991348083

parser = DemoParser("demo.dem")
df = parser.parse_ticks(["total_damage", "mvps", "round"])
df = df[df["steamid"] == wanted_steamid]
df = df.loc[:, ["total_damage", "mvps", "round"]]
df = df.drop_duplicates()

print(df)