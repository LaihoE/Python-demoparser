# CSGO demo parser for Python
Work in progress !


## Game events

```python
parser = PythonDemoParser("demo.dem")
events = parser.parse_events("weapon_fire")
```
## Player data
```python
wanted_props = ["m_vecOrigin", "m_iHealth"]
wanted_players = [76561197991348083]
wanted_ticks = [x for x in range(10000, 11000)]

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
Example player data (output is a df)


```python
    m_iHealth  m_vecOrigin_X  m_vecOrigin_Y  tick            steamid    name
0            1     393.780060       4.702123  1000  76561197991348083  flusha
1            1     394.615204       6.274790  1001  76561197991348083  flusha
2            1     395.472229       7.841670  1002  76561197991348083  flusha
3            1     396.350128       9.403141  1003  76561197991348083  flusha
4            1     397.192352      10.901176  1004  76561197991348083  flusha
..         ...            ...            ...   ...                ...     ...
995          0     918.243347    1071.038330  1995  76561197991348083  flusha
996          0     919.217529    1071.078613  1996  76561197991348083  flusha
997          0     920.191101    1071.131104  1997  76561197991348083  flusha
998          0     920.191101    1071.131104  1998  76561197991348083  flusha
999          0     922.136047    1071.270996  1999  76561197991348083  flusha

[1000 rows x 6 columns]
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
Time taken for the actual parsing:
| Action      | Time |
| ----------- | ---- |
| Game events | 30ms |
| Player data | 1s   |

Numbers are with a ryzen 5900x


As you can see there is a huge difference between time taken for events and props. This is not a surprise since most of data is inside props. The parser only parses the part you are interested in.

Performance still has lots of room for improvement.