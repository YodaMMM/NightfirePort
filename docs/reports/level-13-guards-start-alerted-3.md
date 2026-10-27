# Report: level-13-guards-start-alerted-3

Task: round 2 added the vision bit to PlayScript and the five abseil ENTERs, which fixed eight of
the reds; six remained (escort_follow_07000001_live, walkreal_07000001_live, fix_npc_doors_live,
walkreal_07000008_live, fix_pickups_ammo_live, drone_weapon_drop_live). Round 2's census counted
139 state functions writing `+0x4F8 |= 0x10000`; port the bit on EVERY one of those ENTERs, not
just the ones a red test exposes.

## The census, corrected and pinned

Re-derived from `work/decomp/ACTION.ELF` (one file per function) and pinned to the game's own
state-id mapping — the handler table `NDrone2_StateFuncs` at 0x29b300, read from the EE snapshot
(file offset = EE address) [L ee_snapshot.bin @0x29b300 + id·4]:

- **137 real writers** (`DSTATE_*` functions). The raw grep's 139/160 counts carried three prefix
  false-positives: `Drone_Control` 0x139268:93 writes `|= 0x100000`,
  `DroneVision_AlertSound` 0x177800:70 `|= 0x10000000`, and `KnockedOut_Anim` 0x172a30:21
  `|= 0x1000000` — none writes 0x10000.
- **133 set the bit on ENTER** (the decomp writes it in `case 1:` of the message switch, or under
  `if (msg == 1)` — CombatTooClose 0x173338, CombatNewSighting 0x173280, NinjaSideflip 0x173e08
  use the if-form; every cite with its write line is in the test table below).
- `Attack` 0x164210:69 sets it **on ENTER but conditionally** (the no-attack-route arm, gated on
  `+0xc5 != 2 && != 0x19`). `AllyGoToGoalPosition` 0x15f938:20 (case 3) and `BotGlobal` 0x16e060
  (cases 3/0x2e) set it **on UPDATE, not ENTER**.
- `HeardNoiseAware` 0x16af20:21 sets it **inside the `noise_level < 2` branch only** — the port's
  arm already matched that shape, so it was left alone.

## What changed (crates/nf-game/src/drone.rs)

`vision_on = true` added to **35 more ENTER arms**, each with its [M] cite in the line:
the whole ally/escort lead+follow chain — ALLY_LEAD_INIT 0x15e4f0:24, ALLY_LEAD 0x15e698:21,
PLAYER_IN_WAY 0x15ea28:20, LEAD_HIDE 0x15ec08:18, ALLY_LEAD_WAIT 0x15eda8:20,
LEAD_MISSION_WAIT 0x15efc8:24, LEAD_BOND_COMBAT 0x15f120:19, FOLLOW_INIT 0x15f3c8:16,
FOLLOW 0x15f4e0:21, **FOLLOW_WAIT (new arm) 0x15f6a0:21** — plus COMBAT 0x1731b8:16,
COMBAT_NEW_SIGHTING 0x173280:22, SURRENDER_ANIM 0x1626a0:18, SURRENDERED 0x162950:20,
UNSURRENDER_ANIM 0x162a70:15, STUNNED_RECOVER 0x1630b8:16, STUN_GRENADE_RECOVER 0x163350:16,
STUN_DART_RECOVER 0x163700:20, PUNCH_IMPACT 0x172ee0:16, DRAW_WEAPON 0x166000:23 (the draw path
only, after the CivilianScared bail, as in the game), AIM_STAND_RELOAD|AIM_CROUCH_RELOAD
0x1666d8:16/0x167370:15, AIM_CROUCH 0x166f28:16, TRUCK_DRIVER_MISSION 0x161030:22,
OPEN_DOOR 0x1681f8:19, KICK_OBJECT 0x1684c0:15, ACTION_ANIM 0x168628:16, STAND_FIDDLE 0x1687c0:17,
UNDER_COVER_AIM 0x16a2d0:15, UNDER_COVER_FIRE 0x16a3e0:31 (fire path only, after the sniper bail),
UNDER_COVER_LEAVE 0x173ac0:12, UNDER_COVER_LEAVE_NOW 0x16ab38:15, ASTRONAUT_LAUNCH 0x16dd28:17,
ASTRONAUT_COMBAT 0x174458:16, ASTRONAUT_COMBAT_MOVE 0x174550:18, SPACE_DRAKE 0x16dee0:18
(already had it — verified, not edited). GO_TO_GOAL_ALLY's UPDATE-arm bit and CIVILIAN_MISSION's
ENTER bit were already ported — verified, not edited.

**The census is a test now**: `crates/nf-game/tests/vision_bit_census.rs` (new, ISO-free). It
re-derives every `(STATE, msg::ENTER)` arm from the port source (brace-matched) and requires
`vision_on = true` on ~110 implemented census states, each row carrying its decompile address and
write line. A `NOT_IMPLEMENTED` table pins the 14 writer states the port has no arm for yet
(CastleChatGuard1 0x36, InterogateWalk 0x3d, CombatTooClose 0x5d, Obstructed 0x61,
AimStandDiscard 0x6a, Prone 0x6b, ProneFire 0x6c, HoldItRightThere 0x80, SeenDeadBody 0x9e,
SeenSurrenderedDrone 0x9f, NinjaStand 0xa6, NinjaAttackLong/Mid/ShortRange 0xa8/0xa9/0xaa) — the
moment one grows an arm the test fails until `vision_on` is added and the row moves to REQUIRED.
Special ENTER shapes (BULLET_IMPACT's impact intercept, GRENADE_THROW's `handle_grenade_throw`,
GO_TO_GOAL_ALLY's UPDATE) are checked through their own anchors. Attack's superset and the
noise-family conditionals are documented in the row comments. No existing state constant was
renamed or repurposed; the only new arm (FOLLOW_WAIT) sits beside its UPDATE arm.

## Verification

- `vision_bit_census` — 1 passed (the port matches the census).
- The six reds, through the real binaries (`--test-threads=1`, real ISO):
  escort_follow_07000001_live 1/1, fix_npc_doors_live 1/1, walkreal_07000001_live 3/3 (incl.
  `walk_from_the_spawn_to_the_exit_with_the_contact`), walkreal_07000008_live 4/4, 
  fix_pickups_ammo_live 4/4, drone_weapon_drop_live 3/3 — **all six named tests green**.
- The earlier eight + the 13 start test: guards13_start_live 1/1, exit_aim_01_live 1/1,
  interact_07000001_live 10/10, fix_mission_live 4/4, guard_first_shot_01_live 2/2,
  rappel_vanish_0700004a_live 2/2 — green.
- `nf-game` lib: **578 passed, 0 failed, 5 ignored** (matches round 2's 578/0; two transient
  failures mid-session were the sandbox disk filling up — `StorageFull` in profile save tests —
  and passed again after freeing space; nothing code-side).
- `cargo test --offline --release -j4 --no-run` over all four crates: builds clean.
- **Not run this session: the full four-crate suite end-to-end.** The first detached
  `tools/test/suite.py` start died instantly (its `cargo` Popen got a PATH-less environment), the
  rebuilds + bisects consumed the window, and background jobs are killed between tool calls in
  this sandbox, so the fast runner never got a foreground slot. The AI/escort/walk family the task
  names ran green individually as listed above; nf-formats/nf-audio and the rest of nf-view were
  not re-run.
- Flakiness note: walkreal_07000008_live's first full-binary run (491 s) failed
  `the_ally_fights_only_the_guards_through_the_noise_path` and `walk_the_gondola_ride_to_the_end`;
  both passed in isolation, and the identical binary then passed the full binary 4/4 in 111 s
  (bisected green on the round-2 tree too, so not a regression of this branch — but the
  order/scheduling-dependent variance is worth a look by the main agent).

## What's left

- The 14 NOT_IMPLEMENTED writer states still have no port handler — their sight bit lands when
  the states do (the census test enforces it).
- The full suite re-run on the final tree (the standard pre-merge gate) — next session should
  start `tools/test/suite.py` in the FOREGROUND first thing (this sandbox kills detached jobs).
- walkreal_07000008_live's run-to-run variance (see above).

## Live checks for the main agent

1. Fresh 07000001 boot, the escort NPC: during his route walk, gdbwatch `+0x4F8` (obj+0x608) —
   expect bit 0x10000 set the whole walk now [M 0x15f3c8:16/0x15f4e0:21 family]; his sight ack
   (head toward Bond, alert-status byte) should rise while the walk continues.
2. Same boot, the interior-door guard: watch `+0x4F8` through the OpenDoor state — the game sets
   the bit on that ENTER [M 0x1681f8:19]; the port now runs sight mid-door-open.
3. Fresh 07000008 boot, Zoe (dtype 0xC): during the gondola ride, watch her `+0x4F8` — the
   FOLLOW/LEAD chain now carries the bit in the port; confirm the game's Zoe runs sight while
   riding (the flaky pair above may be her sight pass changing engagement timing).

## Usage

not visible to me (no token/credit/context-window figures are shown in this environment).
