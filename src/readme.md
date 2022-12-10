## Notes on DEMO files


The parser is very bare bones. All unnecessary parts have been stripped out. 
The parser does not create internal abstractions of the demo. For example entities 
are just just a number, nothing more nothing less. Then in the end the parser glues together needed information to infer what player maps to which entity. 



The demo is mostly raw dumps of packets in form of Protobuf messages. There are 3 main ones.

- Game events
- Packet entities
- Stringtables

#### Game events
Interesting points in the demo that trigger this event like when a player is hurt. These are seperate pieces of data that aren't really related to the rest of the demo. These are just 
small dumps of data that could mostly be gathered trough packet entities manually, but are a more elegant way to access some data. The only realtionship these have to other packets, are the ID's, both in form of entity id and user id. These ids don't mean much and need to be mapped to ids in stringtables.

#### Packet entities
Majority of data. All info regarding entities come trough here. The packets contain "deltas" meaning changes in a value. For example players coordinates are only sent when they are moving, not when standing still. This is a massive save in data, but also makes parsing slightly more complicated.


#### Stringtables
All kinds of strange data flow trough here, for example soundprecace and other not so interesting data. The main interesing data that comes trough here are data relating to players like Steamid, Name, Entity id, User id etc.
Default values of packet entitiy props also pass trough here called "instancebaselines". For example when you buy an AK47, packet entities don't send the clip size if there has not been changes to it. That is because it refers to the default value of the mag (30).


### DEMO FORMAT








# other
This means that a value at tick t might have been sent at t-1000. This is a massive save in data, but also makes parsing slightly more complicated. This means that to know what the value should be at tick 50000, we can't just parse the tick 50000, we also need to parse the ticks before. More generally the value at t = value at the latest delta. This means that in theory you could start from tick t and go backwards until you find the delta for that value. 