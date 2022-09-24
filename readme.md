# CSGO demo parser for Python


## Example use

```python
parser = PythonDemoParser("demo.dem")
events = parser.parse_events("weapon_fire")
```

Returns a list of dictionaries with following shape:
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

Returns something like this:
```python
    player_name  event_name round silenced    weapon       tick   player_id
0     flusha  weapon_fire     0    false     weapon_m4a1     146  76561197991348083
1     flusha  weapon_fire     0    false     weapon_m4a1     152  76561197991348083
2     flusha  weapon_fire     0    false     weapon_m4a1     158  76561197991348083
3     flusha  weapon_fire     0    false     weapon_m4a1     164  76561197991348083
4     flusha  weapon_fire     0    false     weapon_m4a1     170  76561197991348083
```



### Game events are only a small part of what is available
Props are values that players can have. For example "m_vecOrigin_X" gives the players X coordinate at a given tick.

```python
props_wanted = ["m_vecOrigin", "m_iHealth"]
parser = PythonDemoParser("demo.dem")
df = parser.parse_props(props_wanted)
```
List of available props can be found here

parse_props also allows optional filtering of steamids and ticks:
```python
wanted_props = ["m_vecOrigin", "m_iHealth"]
wanted_players = [76561197991348083]
wanted_ticks = [44, 88, 132]

parser = PythonDemoParser("demo.dem")
df = parser.parse_props(props_wanted,
                        players=wanted_players,
                        ticks=wanted_ticks)
```



## Performance
**Performance is likely going to improve (especially props speed)**

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

Assuming you are only parsing game events and you have a fast NVME drive you can parse over **24 demos per second** single core and with paralell parsing over **80 demos per second**.