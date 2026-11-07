# agent-lvl-05-06: The Exchange, levels 07000005 and 07000006

Method: every placement, channel and script event of both levels read against the decompile,
then census probes over the disc (`Level::parse` / `nf_view::load`, the 20 story levels).

## Fixed

1. **The upper-door route never completed "Breach the castle walls".** Arriving on 07000006 by
   07000005's cable and door #112 lands on start #511. The only plates of that route, #495/#766
   (ch 0x50), are flat quads 0.68 below a standing Bond. The port's box reach stops at pos − 0.6, so
   OR #498 never raised ch 0x46: the objective stayed open, "Find a way inside to the party" stayed
   hidden, and `Mission_MonitorObjectives`' all-objectives AND could not pass later in the mission.
   The game runs bit-0x40 glists through the raw refine, which reaches past these plates [M 0x194df0,
   0x109da0, 0x1ebaa8:93-99]. New low toucher (`Trigger::raw_plate`): a grounded standing Bond
   directly over a flat bit-0x40 plate, down to about pos − 0.75.
2. **The tunnel plate #524 (ch 0x34) never fired.** Same gap, 0.65 below Bond at the foot of the
   tunnel ramp. It is the plate the upper-door route crosses on the way down to door #71, so the
   courtyard lamps stayed dark on that route until a far plate. Same fix.
3. **Barrel traps sounded at the world origin.** `SsList::tick` read the matrix's w column for the
   position, so sample 283 on #353/#354/#389/#390 played at (0, 0, 0) [M 0x1a9948:86-89, obj+0x80].
   The old test compared against the same wrong index. All levels' SS operate sounds share this.

## Scope of the low toucher

Census over the 20 story levels of flat bit-0x40 plates inside reach: only 06 #495/#766/#524/#745
(#745 shares ch 0x51 with plates that already fire). All 34 `dev::LEVELS` cold starts show the same
channels after 12 ticks with and without the rule. The reach stops short of the full raw radius and
reads grounded Bond only: the full reach fired balcony plate #664 ticks early and moved the 06 walk
pins. The stop is marked [S] in triggers.rs, so 02 #501, 07 #443 and 08 #287 stay on the box.

## Tests

Red on the old behaviour, green now: `arrival_plates_06_live` (2 of 3; the third is the sister),
`ss_operate_sound_06_live`. Green: walkreal 02, 05, 06, 08, interact and interactables 05/06,
searchlight_reveal_06, fix_truck, gate_guard_truck.

## Read and found right

Script events 3-0x13 on both levels, script entities (the truck's AI volume, plates, copter parts),
music updaters, the 0x6000994 music gate, level-gated drone code (ReactionTime, FirstAlertState,
CheckAlarmRaised, the hurt class gate, DefaultInit's 05/06 props), loadouts, Bond moments, copter
spot latch (p8 = 100 turns it off on 05), spawners, SS state machine, room ambient channels.

## Not fixed

- Window_Securityroom #376 is not solid to Bond. It is the one pane on 06, sill about 0.75 up.
  `Collide_PickObj` masks by query: glist flag 0x1000 passes sight rays while the walk query stops
  at the pane [M 0x1eae80:72-82]. The port's parts set has no per-query mask, which is why panes are
  held out on every level.
- Off these levels: `ally::on_init_death` has the killer inverted. `bVar7` is class 3 = the player
  [M 0x145bd0:17-20, Player_Init obj+0xff = 3], the port reads it as a drone. Prop-0x54-only actors
  (01, 02, 07, 08, 12, 16) fail on a drone kill and not on a player kill, the reverse of the game.
  The 06 driver carries both props, so 06 is unaffected. `set_death_channel` also drops channel 0x63
  [M 0x13e660]; on 06 no path reaches it without the props firing first.

## Unproven

Whether the game fires #495/#766 on the spawn tick or a tick later: read from code, not captured.
