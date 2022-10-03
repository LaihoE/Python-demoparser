# CSGO demo parser for Python

```python
pip install demoparser
```

## Game events

```python
parser = PythonDemoParser("demo.dem")
events = parser.parse_events("weapon_fire")
```
## Player data
```python
wanted_props = ["m_vecOrigin", "m_iHealth"]
wanted_players = [76561197991348083]
wanted_ticks = [44, 88, 132]

parser = PythonDemoParser("demo.dem")
df = parser.parse_props(props_wanted,
                        players=wanted_players,
                        ticks=wanted_ticks)
```

Example game event
```python
{
'player_name': 'flusha',
'event_name': 'weapon_fire',
'round': '0',
'silenced': 'false',
'weapon': 'weapon_ak47',
'tick': '18',
'player_id': '76561197991348083'
}
```

Or if you prefer Pandas Dataframes
```python
parser = PythonDemoParser("demo.dem")
df = parser.parse_events("weapon_fire", format="df")
```





## Performance

Performance can be split in two parts. Reading the demo and parsing the demo. 
Performance will vary mostly based on reading speed.

For reference here are some very rough numbers for reading speeds assuming an average demo size of 80MB.
### Reading
| Drive            | Read Speed | Time one demo | Demos/second |
| ---------------- | ---------- | ------------- | ------------ |
| HDD              | 100 MB/s   | 0.8s          | 1.25         |
| Normal SSD       | 500 MB/s   | 0.160s        | 6.25         |
| Average nvme SSD | 3000 MB/s  | 0.026s        | 37.5         |
| Fast nvme SSD    | 7000 MB/s  | 0.0114s       | 87.5         |

### Parsing
So this is how fast reading the raw data takes, then for parsing:
| Action      | Time  |
| ----------- | ----- |
| Game events | 30ms  |
| Props       | 500ms |

As you can see there is a huge difference between time taken for events and props. This is not a surprise since most of data is inside props. The parser only parses the part you are interested in.

Performance still has lots of room for improvement