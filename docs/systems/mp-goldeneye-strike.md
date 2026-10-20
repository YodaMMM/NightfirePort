# mp-goldeneye-strike: the GoldenEye strike's timeline, from the game's own script

The MP GoldenEye scenario's strike (the satellite beam that kills a target after both items are
carried by one team) is driven by a ScriptPlayer — `SP_Create(pos, 0, 0x600003e, 0, 1, 1, 0, 0)`
— whose script's own length is the flight time. The port flew the strike for a 3 s stand-in
([S], the old `GE_STRIKE_FRAMES = 180`); the game's timeline is the script. This page pins the
chain with addresses; the port now reads the length from the arena's own script and tracks the
strike's position the way `MP_GoldenEyeUpdate` does.

## The launch ([M] `MP_GoldenEyeUpdate` 0x187568:356-401)

Both GoldenEye items carried by one team (`+0xf4 == 1` on both and the team bytes equal,
0x187568:356-361) → `MP_GetTarget(carrying team == 0, 0, 1)` picks a random live player of the
opposing team into the strike's target global (`DAT_003181a0`, :362-367; no target = no launch,
:365-366); each carrier's match score +1.0 (:368-377); the launch sound 0x14a (:378); both items
go to state 3 with the `+0xf0` bit 0x10 (:379-382); and the strike ScriptPlayer is created at the
carried item obj's `+0x30` position with the script `0x600003e` (= 0x0600003e), speed config word
0 → 1.0 (:383-384, `SP_Create` 0x1dc7e0 → `SP_CreateScriptPlayer` 0x1db9a8). The HUD message
(0x2000042) and the bot target's camera mode 1 (:385-398) follow.

## The script ([D] `07000023.bin` entry `0600003e`, 416 bytes)

`nf_formats::nis::Nis` layout (magic 0x1C): 2 streams, loop start 0, **length 299 ticks** —
4.983 s at the fixed 60 Hz. The stream commands (decoded with the port's `Stream::decode`):
stream 0 creates the strike entity (hash 0x37030002, obj type 0x31) at marker 0 and ends it at
marker 11009 with the paired `End`; stream 1 runs a `Light` (parent none, kind 0) through the
same span — the beam and its light. The keys carry only local offsets (scale 0.1/1.0/6.0): the
ScriptPlayer object itself never moves from its keys — its world position is re-set every tick by
`MP_GoldenEyeUpdate` (below). Every GoldenEye arena carries its own copy (the hash recurs across
the level archives [D]); 07000023's length is 299.

## The flight ([M] `SP_CreateScriptPlayer` 0x1db9a8 → `SP_Update` 0x1dbda8 → `SP_GetState` 0x1dc900)

- Created with the config `{+0x2c: script 0x0600003e, +0x34: 1, +0x38: 1, +0x3c: 1}`: the
  SCRIPTPLAYER state `+0x24` starts 6 then 7 (+0x2e == 0, then +0x26 == 1), and `SP_LoadScript`
  0x1dbc08 (`Script_Load` 0x1d8248 → the script hashtable; `Script_Play(info, 0)`) sets state 2 —
  playing.
- `SP_Update` case 2 ticks the script (`Script_Update`); when `Script_IsPlaying` 0x1db630 reads
  the SCRIPTINFO `+0xab2` play byte done (bit 1 — the clock past the length; the port's
  `nis::Script` models the same byte as `play: 1 playing, 2 done, 8 looping`), the `+0x35` arm
  (= config `+0x3c` = 1) sends the state to 5 (:248-251). `SP_GetState` returns 2 exactly when
  the state is 5 or 3 (:10-20) — **the game's "strike landed" test**.
- So the flight lasts the script's length in ticks: 299 on 07000023, from the launch tick
  ([S ±1 — the SP's object pass and the MP pass order within one frame is not read]).

**The strike tracks its target every tick** ([M] 0x187568:87-95): at the top of every
`MP_GoldenEyeUpdate`, while the strike ScriptPlayer exists and the target pointer is set, the
target obj's position vector (`+0x30..+0x3c`) with **y − 1.0** is written into the SP
(`SP_SetPos` 0x1dc990) and the redraw flag `+0xfa |= 0x20` set. The beam sits on the target
wherever the target goes — the launch position (the item obj's `+0x30`) only lives until the
first update tick.

## The impact ([M] 0x187568:96-179)

`SP_GetState == 2`: the target (if `MP_playerIsDead() == 0`, :99-109) dies — a bot
(`+0xff == 3`) through `Player_Kill`, a human through `BLData+0xac = -1.0` (the drone health
field, [M 0x187568:105-108]; `Drone_SM_SetState(BL+0x108, 0xf2)`). The target's opposing team
scores +1.0 (`MPGame + 0x180/0x184` by the target's team word, :110-117 — `Control_Plr2Ind`'s
index) and the target's status word = 0xfffe (:118). The strike and target globals clear
(:121-122); `MP_setPlayerStatus(.., 4/8, 0x37/0x38, ..)` posts the scenario statuses (:123-126);
both items re-pick a random pool spawn (`Rand_Rand(GoldenEyeSpawnCounts[type != 6])`, the
matrix copies, state 0, unlink/rebuild) (:129-179). No area damage: the only wound is the
target's [M :98-108].

## The port (this task)

- `MpObjectsRt.ge_flight_ticks: u16` — the flight length, filled by the live load from the
  arena's own script: `level.nis.get(0x0600_003e).map(|n| n.length)` (299 on 07000023 [D]).
  0 = no script in the arena: the game's ScriptPlayer never plays and the strike never lands —
  the port matches (the items stay in state 3), exactly the game's own soft-lock [M
  Script_Load 0x1d8248's failed-lookup arm]. The headless rigs set it directly when they drive a
  strike.
- `MpObjectsRt.ge_strike_pos: Option<[f32; 3]>` — the tracked beam position, hashed with the
  target (folded inside `hash_state`'s `ge_target` Some arm, so the non-GE scenarios' hash bytes
  are unchanged): Some(the item's pos) at the launch [M :383-384], Some((target.x, target.y − 1.0,
  target.z)) on every in-flight tick [M :87-95], None at the impact [M :121-122].
- The completion check reads `ge_flight_ticks` (0 → never) instead of the 180 stand-in.
- The draw getters ([`MpMatch::ge_strike_view`]): `None` when no strike flies; otherwise
  `{ target, pos, ticks_left }` — the beam position (the tracked value above), the target slot,
  and the ticks left of the script. The blast renders at the last tracked position on the tick
  the view vanishes (the game's own visuals ride the script's entity/light streams, already in
  `level.nis` for the renderer).
- The kill path is unchanged (`ge_strike_complete` → `player_killed` through the shared combat —
  the hashed death/score/status path the existing tests pin [M :98-118]).

## Tests (`crates/nf-view/tests/mp_scenarios_live.rs`, NF_ISO-gated, the hashes agree every tick)

- `goldeneye_online` — updated: the flight bounds read `ge_flight_ticks` (299 on 07000023) with
  the ±2 tolerance [S the SP/MP pass order]; during the flight the strike's position equals the
  parked target's `(x, y − 1.0, z)` on both machines [M :87-95]; the existing pins (the launch
  scores, the death on both machines, the item re-picks, the respawn) unchanged.
- The sister: every other scenario's test in the same binary and the deathmatch digest pin
  (`mp_perf_16_live`) stay green — the hash fold change is inside the `ge_target` Some arm, so
  non-GE matches hash byte-identically.
