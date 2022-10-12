# CSGO demo parser for Python
Work in progress! expect some bugs here and there
## Installing
```python
pip install demoparser
```

## Game events

```python
from demoparser import DemoParser

parser = DemoParser("path_to_demo.dem")
events = parser.parse_events("player_death")
```
## Player data
```python
from demoparser import DemoParser

wanted_props = ["m_vecOrigin_X", "m_iHealth"]

parser = DemoParser("path_to_demo.dem")
df = parser.parse_props(wanted_props)
```
parse_props also accepts optional arguments ticks and players like so:
## Player data
```python
from demoparser import DemoParser

wanted_props = ["m_vecOrigin_X", "m_iHealth"]
players = [76561197991348083]
ticks = [768, 897, 1848, 9443]

parser = DemoParser("path_to_demo.dem")
df = parser.parse_props(wanted_props, players=players, ticks=ticks)
```

#### Example game event
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
List of possible events: [GameEvents](https://wiki.alliedmods.net/Counter-Strike:_Global_Offensive_Events)
#### Example player data (output is a df)


```python
       m_iHealth  m_vecOrigin_X  m_vecOrigin_Y    tick            steamid    name
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
Player data uses the real names for the props with one exception. All "vector" props are rolled out into their own columns. For example:
m_vecOrigin --> (m_vecOrigin_X, m_vecOrigin_Y).   
List of possible values comming soon.
## Performance

Your performance will mostly depend on how fast your HDD/SSD is. Below are some rough estimates for parsing speeds excluding I/O.




| Action      | Time  |
| ----------- | ----- |
| Game events | 50ms  |
| Player data | 250ms |

Time taken for the parsing (with ryzen 5900x and no I/O):


## Other notes
- Parser uses mmap.

