# On demo parsing

The demo is mostly a raw dump of packets in form of Protobuf messages. The 3 main ones are:

- Game events
- Packet entities
- Stringtables

#### Game events
Interesting points in the demo like when a player is hurt. These are seperate pieces of data that aren't really related to the rest of the demo. These are just small dumps of data that could mostly be gathered trough packet entities manually, but are a more elegant way to access some data. The only realtionship these have to other messages, are the ID's, both in form of entity id and user id. These ids don't mean much and need to be mapped to ids in stringtables.

#### Packet entities
Majority of data. All info regarding entities come trough here. Almost the entire message is in a field called entity_data that is a long steam of bits. Noteworthy is that packet entities only include "deltas" meaning when a value changes.

#### Stringtables
All kinds of strange data flow trough here, for example soundprecace and other not so interesting data. The main interesing data that comes trough here are data relating to players like Steamid, Name, Entity id, User id etc.
Default values of packet entitiy props also pass trough here called "instancebaselines". For example when you buy an AK47, packet entities don't send the clip size if there has not been changes to it. That is because it refers to the default value of the mag (30).
