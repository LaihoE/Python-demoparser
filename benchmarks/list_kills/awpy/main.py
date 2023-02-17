from awpy.parser import DemoParser
from glob import glob
from tqdm import tqdm

files = glob("/home/laiho/Documents/demos/bench_pro_demos/*")
for file in tqdm(files):
    demo_parser = DemoParser(
        demofile = file, 
        parse_rate=128, 
        trade_time=1, 
    )

    df = demo_parser.parse(return_type="df")
    print(df["kills"])