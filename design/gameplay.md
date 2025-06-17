What do you actually do?
------------------------

# GuildMaster
The primary interface is as the guildmaster where you raise funds and develop
your Guild in order to raise enough funds to stick it to the rival guild accross
the street that is run by a complete arse.
This involves designing your guild layout, purchasing resources and training
members. When they are sent on a mission the result is instant (fuck timers) due
to the strange quantum uncertainty of the archive.

That and the fact that they'll come back the next morning.
While this is great for business it's rather bad for the health, sanity and
temporal rigidity of the guild members involved.

Initially you will have a single room in the basement of the town pub and you
will quickly be able to buy out the pub (following the mysterious death of the
current owner following your first outing). From there you can build more rooms,
buy more buildings and design your guild to your heart's content. (Well, so long
as I've programmed that part...)


# GuildMember
If you thirst for adventure yourself you can take a bar bet with the guild
Wizard (once you unlock him) and posess the next guild member who applies in
order to take a turn in the Archive (why on _Earth_ would you go in there
yourself? Are you insain?!)

This is a one time deal that will - sadly - turn the mind of the guild member in
to jelly at the end of the adventure. Lime jelly to be precise. But, for a brief
and shining moment, they will have their hand guided by your knowing...hand.

_NOTE::_ If you manage to find a potion of _retain sanity_ on your adventures
then your chosen vessel will - unsurprisingly - retain the use of their
executive functions on return _and_ get a large XP boost too boot!

Gameplay in this mode is more like a traditional roguelike with the BoI style of
dungeon that is a connected set of rooms only as long boring corridors are...
well, boring.


### GuildMaster concepts
- Hiring staff
- Funding expeditions
- Recruitment posters / talent scouts
- Building the guildhouse
- Getting stock (food, drink, default adventurers kit)
- Setting member contracts and perks
- Planning and running adventures

------------------------------------------------------------------------------
# Player stats
STR: damage mod, crit damage
DEX: hit, evasion, crit chance
VIT: hp, damage reduction, regen rate
ENL: focus, favour rate

### Remove races and only have classes and objects of worship
# Classes
Thief {2,4,3,3}
- +1 to hit and damage with daggers and bows
- can only wear light armour
- hidden attack: spend favour to do double damage to enemies that have not detected you

Priest {3,1,3,5}
- improved favour rate
- can only use crushing weapons
- righteous smite: spend favour to do extra damage

Brawler {5,2,4,1}
- all weapon die rolls have an extra side
- no ranged weapons
- overkill: 10 favour for each extra point of damage on kill

Mage {2,2,2,6}
- no armour
- starts with two spells
- recharge: spend favour to recharge spells


# Item stats
The general rule (taken from Sil): (offence) [defence]

> Weapons
---------
(hitMod, damageDice) [evMod]
i.e.
  Long Sword: (+1, 2d6) [+1]
  Short Sword (0, 1d8)  [+2]
  Dagger      (+1, 1d6) [+1]

> Armour
--------
(hitMod) [evMod, absorb]
i.e.
  Leather (0)  [0, 1d4]
  Studded (0)  [-1, 1d5]
  Mail    (-2) [-2, 2d4]
