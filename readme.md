# CSGO demo parser for Python

Demo parser for Counter-Strike: Global Offensive. Parser is used to collect data from replay files (".dem" files).  
The goal of the parser fast and simple. Performance is solved by having Rust do the heavy lifting and to keep it simple we completely avoid "hooks" and rather just let users "query" the demo.

## Installing
```bash
pip install demoparser
```

## Game events

```python
from demoparser import DemoParser

parser = DemoParser("path_to_demo.dem")
events = parser.parse_events("player_death")
```
List of possible events: [GameEvents](https://wiki.alliedmods.net/Counter-Strike:_Global_Offensive_Events)  
WARNING all demos do not contain every possible event.


Events can easily be transformed into a df:
```python
df = pd.DataFrame(events)
```
You can also add "tick data" into game events like so:
```python
events = parser.parse_events("player_death", props=["X", "Y", "health"])
```
## Tick data
```python
from demoparser import DemoParser

wanted_props = ["X", "Y", "Z", "health"]

parser = DemoParser("path_to_demo.dem")
df = parser.parse_ticks(wanted_props)
```
List of possible props:[props](https://github.com/LaihoE/Python-demoparser/blob/main/vars.md)  
parse_ticks also accepts optional arguments for filtering players and ticks:

```python
df = parser.parse_ticks(wanted_props, players=[76511958412365], ticks=[489, 5884])
```


## Utility functions
there are also these 2 functions for header and player info. Takes no arguments and returns some data.
```python
parser.parse_header()
parser.parse_players()
```
#### Example game event
```python
{
'player_name': 'flusha',
'event_name': 'weapon_fire',
'round': 0,
'silenced': 'false',
'weapon': 'weapon_ak47',
'tick': 18,
'player_id': 76561197991348083
}
```

#### Example tick data (output is a df)


```python
           health         X              Y        tick        steamid       name
0            100     148.867508    -413.923218   10000  76561197991348083  flusha
1            100     149.625168    -412.063995   10001  76561197991348083  flusha
2            100     150.342468    -410.183685   10002  76561197991348083  flusha
3            100     151.025726    -408.286407   10003  76561197991348083  flusha
4            100     151.677643    -406.374207   10004  76561197991348083  flusha
...          ...            ...            ...     ...                ...     ...
90911         86   -1684.031250    2547.948975  100995  76561197991348083  flusha
90912         86   -1684.031250    2547.948975  100996  76561197991348083  flusha
90913         86   -1684.031250    2547.948975  100997  76561197991348083  flusha
90914         86   -1684.031250    2547.948975  100998  76561197991348083  flusha
90915         86   -1684.031250    2547.948975  100999  76561197991348083  flusha

[90916 rows x 6 columns]
```

## Performance

**Your performance will mostly depend on how fast your HDD/SSD is.** You can roughly calculate this part by demo size / reading speed of your drive.

Below are some rough estimates for parsing speeds **excluding I/O**. 


| Action                             | Time      |
| ---------------------------------- | --------- |
| Game events with no "tick data"    | 30ms      |
| Game events with "tick data"       | 310ms     |
| Tick data: 1 value (no early exit) | 300ms     |
| Tick data: 5 million values        | 800ms     |
| Rust only                          | 150-200ms |

Time taken for the parsing (with ryzen 5900x and no I/O):

If you have a fast SSD then i strongly recommend multiprocessing your parsing. [Examples](https://github.com/LaihoE/Python-demoparser/tree/main/examples) show how to multiprocess across demos. Multiprocessing will most likely max out your drive's reading speed. With multiprocessing ive been able to parse > 5GB/s (of game events) and >3GB/s (tick data). An average MM demo is around 90MB.

Performance will definetly still continue to improve, especially tick data with big number of values.
"Rust only" means one full parse of the demo, compareable to what other parsers are doing. This part is probably quite close to the limit on how fast we can go, but we might be able to partially skip data, leading to even bigger improvements, but parsing every tick is not likely to improve by much.

Current flamegraph of performance: [flamegraph](https://github.com/LaihoE/Python-demoparser/blob/main/flamegraph.svg). Save the image and open it in a browser to zoom.

## Other notes
- Demo tickrate is not the same as server tickrate. Often demo tickrate is half of server tickrate. For example faceit demos are 64 tick and MM demos are 32 tick. This means that every other tick is "missing". Pro games are often recorded at native tickrate.
- First and last ticks often have many NaN values. For example if player isn't connected this happens.
- Game events have lots of information. Look there first.
- Exact granade trajectories are currently not supported. What you can find is where the granade was thrown from ("weapon_fire" event) and where it detonated (for example "hegrenade_detonate" event). Detonate event also includes coordinates but the weapon fire does not.
- Parser early exits when biggest wanted tick is reached
- If using WSL make sure the demos are also on WSL (or it will be slower)


## Why yet another parser?
Currently you have to take such a big performance hit if you want to use Python, that most people just go elsewhere like Markus-wa's [GO parser](https://github.com/markus-wa/demoinfocs-golang) (great alternative if you want something mature). Unfortunately GO is just not such a popular language and most data analysis is done in Python/R. I also expect that most people interested in parsing demos are not experienced programmers and these people are very unlikely to learn a new language just for demo parsing. Demo parsing is something I think should be doable for very unexperienced programmers. 

The parser is completely written in Rust (same speed as C/C++), (memory safe btw). This leaves the door open for the parser to become more or less as fast as we can go.

Also this type of setup makes it easy to create bindings for other languages (mainly R). Maybe in the future?

## Special thank you to
Valve for "official" implementation https://github.com/ValveSoftware/csgo-demoinfo  
Markuswa's Go parser: https://github.com/markus-wa/demoinfocs-golang  
Saul's: JS parser: https://github.com/saul/demofile  
akiver Demos-Manager (really handy for quickly looking at demos) https://github.com/akiver/CSGO-Demos-Manager  