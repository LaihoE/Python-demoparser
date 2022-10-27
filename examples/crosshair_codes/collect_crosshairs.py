from demoparser import DemoParser
import pandas as pd

parser = DemoParser("demo.dem")
df = pd.DataFrame(parser.parse_players())
print(df.loc[:, ["name", "steamid", "crosshair_code"]])
