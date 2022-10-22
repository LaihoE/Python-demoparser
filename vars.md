Some example props that can be gotten from players.

|         Name          | Real name                            | Desc                                                                                                           |
| :-------------------: | :----------------------------------- | :------------------------------------------------------------------------------------------------------------- |
|           X           | m_vecOrigin                          | X coordinate                                                                                                   |
|           Y           | m_vecOrigin                          | Y coordinate                                                                                                   |
|           Z           | m_vecOrigin[2]                       | Z coordinate  (WARNING is the position of players feet? not head height)                                       |
|      velocity_X       | m_vecVelocity[0]                     | X Velocity                                                                                                     |
|      velocity_Y       | m_vecVelocity[1]                     | Y Velocity                                                                                                     |
|      velocity_Z       | m_vecVelocity[2]                     | Z Velocity                                                                                                     |
|     viewangle_yaw     | m_angEyeAngles[1]                    | Yaw or "how many degrees the player is looking in the right-left direction"                                    |
|    viewangle_pitch    | m_angEyeAngles[0]                    | Pitch or "how many degrees the player is looking in the up-down direction"                                     |
|        ducked         | m_bDucked                            | Is player ducked                                                                                               |
|        scoped         | m_bIsScoped                          | Is player scoped                                                                                               |  |
|        health         | m_iHealth                            | Players health                                                                                                 |
|      in_buy_zone      | m_bInBuyZone                         | Is player in buy zone                                                                                          |
|    flash_duration     | m_flFlashDuration                    | How many seconds the player is blind for. Big value = very blind :D. 0 = not blind                             |
|        balance        | m_iAccount                           | Players current balance                                                                                        |
|         ping          | m_iPing                              | Players ping or "latency"                                                                                      |
|         score         | m_iScore                             | Score                                                                                                          |
|        deaths         | m_iDeaths                            | Deaths                                                                                                         |
|         kills         | m_iKills                             | Kills                                                                                                          |
|        assists        | m_iAssists                           | Assists                                                                                                        |
|         mvps          | m_iMVPs                              | MVPs                                                                                                           |
|         armor         | m_iArmor                             | Armor value                                                                                                    |
|      silencer_on      | m_bSilencerOn                        | Player has silencer on                                                                                         |
|      place_name       | m_szLastPlaceName                    | Human reader place name like "ramp" or "palace"                                                                |
| total_enemies_flashed | m_iMatchStats_EnemiesFlashed_Total   | Total number of enemies flashed during the entire game so far                                                  |
|   total_util_damage   | m_iMatchStats_UtilityDamage_Total    | Total number of utiliy during the entire game so far                                                           |
|   total_cash_earned   | m_iMatchStats_CashEarned_Total       | Total cash earned curing the entire game so far                                                                |
| total_objective_total | m_iMatchStats_Objective_Total        | ?                                                                                                              |
|    total_headshots    | m_iMatchStats_HeadShotKills_Total    | Total number of headshot kills during the entire game    so far                                                |
|     total_assists     | m_iMatchStats_Assists_Total          | Total number of assists duiring the entire game so far                                                         |
|     total_deaths      | m_iMatchStats_Deaths_Total           | Total number of deaths during the game so far                                                                  |
|    total_live_time    | m_iMatchStats_LiveTime_Total         | Total time alive during the demo so far                                                                        |
|   total_kill_reward   | m_iMatchStats_KillReward_Total       | Total money got as kill-reward during the demo so far                                                          |
| total_equipment_value | m_iMatchStats_EquipmentValue_Total   | Total                                                                                                          |
|     total_damage      | m_iMatchStats_Damage_Total           | Total damage during entire game so far                                                                         |
|          3ks          | m_iMatchStats_3k_Total               | Total number of rounds with 3 kills during the game so far                                                     |
|          4ks          | m_iMatchStats_4k_Total               | Total number of rounds with 4 kills during the game so far                                                     |
|          5ks          | m_iMatchStats_5k_Total               | Total number of rounds with 5 kills during the game so far                                                     |
|      total_kills      | m_iMatchStats_Kills_Total            | Total kills during entire game so far                                                                          |
|     crosshaircode     | m_szCrosshairCodes                   | Crosshair code, can be used to get players crosshair                                                           |
|     is_auto_muted     | m_bHasCommunicationAbuseMute         | Is player reported enough to be automatically muted                                                            |
|    friendly_honors    | m_nPersonaDataPublicCommendsFriendly | number of "friendly" honors                                                                                    |
|    teacher_honors     | m_nPersonaDataPublicCommendsTeacher  | number of "teacher" honors                                                                                     |
|     leader_honors     | m_nPersonaDataPublicCommendsLeader   | number of "leader" honors                                                                                      |
|     public_level      | m_nPersonaDataPublicLevel            | ?                                                                                                              |
|       music_kit       | m_nMusicID                           | id for the music kit player uses (music played when player gets an MVP)                                        |
|   active_coin_rank    | m_nActiveCoinRank                    | ?                                                                                                              |
| cash_spent_this_round | m_iCashSpentThisRound                | Cash spent this round                                                                                          |
|   total_cash_spent    | m_iTotalCashSpent                    | total cash spent during entire game? (not so far)                                                              |
|         clan          | m_szClan                             | Clan name (group displayed before players name)                                                                |
| controlled_by_player  | m_iControlledByPlayer                | Something related to bot controlling? I guess not used normally anymore                                        |
|   controlled_player   | m_iControlledPlayer                  | Something related to bot controlling? I guess not used normally anymore                                        |
|    controlling_bot    | m_bControllingBot                    | Something related to bot controlling? I guess not used normally anymore                                        |
|    lifetime_start     | m_iLifetimeStart                     | When did the player spawn last time (measured as seconds since demo started)                                   |
|     lifetime_end      | m_iLifetimeEnd                       | When did the player die last time (measured as seconds since demo started) -1 if the player is currently alive |  |
|         color         | m_iCompTeammateColor                 | Players color, for example "purple"                                                                            |
|       connected       | m_bConnected                         | Is the player currently connected                                                                              |


Parser also allows you to use the "real names". Just make sure you add _X _Y and potentially _Z to the vector prop names. Like so:
m_vecOrigin -> m_vecOrigin_X for x coordinate.