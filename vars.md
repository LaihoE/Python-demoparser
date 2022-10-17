Some example props that can be gotten from players.

|      Name       | Real name         | Desc                                                                               |
| :-------------: | :---------------- | :--------------------------------------------------------------------------------- |
|        X        | m_vecOrigin       | X coordinate                                                                       |
|        Y        | m_vecOrigin       | Y coordinate                                                                       |
|        Z        | m_vecOrigin[2]    | Z coordinate  (WARNING is the position of players feet? not head height)           |
|   velocity_X    | m_vecVelocity[0]  | X Velocity                                                                         |
|   velocity_Y    | m_vecVelocity[1]  | Y Velocity                                                                         |
|   velocity_Z    | m_vecVelocity[2]  | Z Velocity                                                                         |
|  viewangle_yaw  | m_angEyeAngles[1] | Yaw or "how many degrees the player is looking in the right-left direction"        |
| viewangle_pitch | m_angEyeAngles[0] | pitch or "how many degrees the player is looking in the up-down direction"         |
|     ducked      | m_bDucked         | is player   ducked                                                                 |
|     scoped      | m_bIsScoped       | is player scoped                                                                   |  |
|     health      | m_iHealth         | players health                                                                     |
|   in_buy_zone   | m_bInBuyZone      | is player in buy zone                                                              |
| flash_duration  | m_flFlashDuration | How many seconds the player is blind for. Big value = very blind :D. 0 = not blind |



Parser also allows you to use the "real names". Just make sure you add _X _Y and potentially _X to the vector prop names. Like so:
m_vecOrigin -> m_vecOrigin_X for x coordinate.