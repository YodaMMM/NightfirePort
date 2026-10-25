# level-07-guest-collision-5 — the per-cause round: the broadcast cone landed, the full push measured and reverted

Branch `s/level-07-guest-collision-5` = `origin/s/level-07-guest-collision-4` (5e77c07b) merged with
`origin/master` (c528468b), clean (e668ccdf). WORKLOG entry `level-07-guest-collision-5` in
`docs/WORKLOG.md`.

## What was done

Setup: the sandbox kept the repo, `work/` and the ISO; the remote token was refreshed, the branch
created and pushed with the WORKLOG entry first. Two environment notes for the next round: the
background `tools/test/suite.py` dies between tool calls here (same as round 4), and the sandbox
rootfs hit 100% mid-build — 5.2 G of stale test executables in `target/release/deps` (every binary
relinks on demand) plus a stale 4.8 G `/tmp/my-project` copy were deleted to unblock the linker
(SIGBUS in `ld` when the rootfs is full).

### Item 2 — the explosive broadcast gate (LANDED, commit 0846a1d4)

`DroneFunc_HandleExplosives` 0x1435f0 requires, per drone: the fear props (0x3A/0x3B), the radius,
and `DroneVision_ObjectVisibility ≥ 0.7` + `CanSeeObject(5,0)` [M 0x1435f0:30-44]. The port had the
props and the radius but stood the visibility check in with a **bare segment LOS** — a documented
deviation. The cone is now ported into the drain (`crates/nf-game/src/drone.rs`, the
`explosive_queue` drain): ObjectVisibility 0x1781a8 returns exactly 0.0/1.0, so "≥ 0.7" is the pass
bit of — dist < 2.0 always visible; dist > 50.0 never; else `|angle_difference(+0x380 facing,
atan2(dx,dz))| ≤ +0xe4` (when e4 ≠ 0) AND dist ≤ +0xe8 [M 0x1781a8:30-48]. The port's `yaw` /
`half_fov` / `sight_range` are +0x380/+0xe4/+0xe8 — the same expression the FindAlertedDrones alert
pass already runs (the port's own cone port), now reused here. `CanSeeObject(5,0)` keeps the
head-ray segment test.

Diagnostics for the "which grenade bounced" open item: `NF_AILOG` now prints `[expl] queue push
p=… wt=… bounces=…` at the push and `[expl] drain …` plus per-drone `cone`/`los` verdicts in the
drain. On the green run below the queue stayed empty (no `[expl]` lines) — the round-4 trace's
msg 0x21 needs that log on a red run to identify the bounced object; the next round should re-run
the repro with `NF_AILOG=1` if the red ever returns.

**Verification [L, this port run]:** `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`
**GREEN** (231.56 s, `--test-threads=1`) — the round-4 red, where the contact wedged into
CIVILIAN_SCARED → HIDING 0x13 through the spurious 0x21 chain. `the_plate_refuses_bond_alone`
GREEN, `the_hall_guards_wait_for_the_meet_scenes_channel` GREEN.

### Item 3 — the full push: implemented, verified, REVERTED (no code change kept)

The round-1 [S] airborne-only gate for non-guest bots was removed (`player.rs`: every capsule-ON
bot pushes grounded Bond in every stance [M NDrone2_DoCollision 0x149ec8:11-34 — guests
unconditionally, the other classes through the NearPlayerCollision 0x14b190 near/OR arms within
2.0; Collide_Jointy 0x1ec180:229-266 pushes the class-3 player with the full overlap, no stance
gate in Player_Collision 0x194df0]), and the walker_b2 bot-dodge was mirrored to the same
pusher set. The mechanism works — and it **regresses master-green**
`walkreal_07000007_live::walk_the_party_start_to_finish` (STUCK at the hall door, d=1.46, no world
segment): the blocker census [L, NF_PUSH_DEBUG] shows exactly the two TEAM-1 CivilianDoorGuards
(drones 10/19) at spines (94.66, 44.40→43.40, -47.96) and (93.88, 44.40→43.40, -47.90), r=0.4 each
— their centres are 0.78 apart, so their capsules **overlap across the hall-door throat**: no lane
inside the 0.95 hold exists, and the dodge cannot clear it against the hall wall. In the real game
the same push applies [M], so the real player's route must go around them a different way — that is
exactly collision-4's standing live check #4 (which way does the real player's corner leg go).
Per the "nothing red that is green on master" rule both edits were **reverted**
(`git restore player.rs walker_b2.rs`); the census and the geometry stay in this report and the
WORKLOG for the next round.

The elevator_09 census [L, NF_PUSH_DEBUG]: **no bot capsule at the cab stall** — the last pusher in
the log sits at (1.60, 22.30) while Bond stalls at (-4.76, 12.02), 1.9+ away. The cab-mouth
blocker (d=1.00, no world segment) is therefore NOT in the pusher set; it lives in the
trigger/geometry layer. The faithful push would not have fixed it; next round needs a trigger
census at the stall or a live check.

### Item 1 — the slide's boundary consumer (decomp re-read, no code change)

The clamp consumer is confirmed [M Drone_Control 0x139268:69-88]: the slide arm runs only when
`NDrone2_DoAnimation` returned 0 and `+0x580 != GameState._52_4_`; the clamp needs `+0x4f8 & 1`
AND `+0xba8 == 0`, then boundary bits 2/4 (local ±z probes, `DroneMove_SetBoundryFlags` 0x14f140 —
feet ±0.5 through `NDrone2_MoveTest`, bit set when the probe != 1) zero the positive/negative
slide; bits 8/0x10 gate the PreTransCallback root motion [M 0x13b490:120-135]. The round-4
conclusion stands: three clamp variants were each tried and reverted there, and the distinguishing
mechanism is the game's route-stall danim swap (the 0x63 blocked arm) which stops the game's slide
with the route. That state-machine port did not fit the window after the push round; the port's
raw add is unchanged from the merged tree and `walkreal_07000007_live` is 3/3 GREEN on it.

### Item 4 — armour_0a, speech_facial

Not addressed this round (the push investigation consumed the window). Both diagnoses stand as
collision-4 wrote them.

## Verification performed this session

All runs `--test-threads=1`, `NF_SILENT=1`, on the final tree (0846a1d4 = the merge + the cone
commit; the push edits reverted):

| binary | result |
|---|---|
| `walkreal_07000001_live` | **3/3 GREEN** (the main contact walk 231.56 s — the round-4 red fixed) |
| `walkreal_07000007_live` | 3/3 GREEN (re-verified after the revert) |
| `fix_guest_solid_live` | 3/3 GREEN |
| `sleeper_push_live` | 1/1 GREEN |
| `npc_doors_live` | 5 passed / 1 FAILED / 1 ignored — the round-4 pre-existing state (the shoji red, item 1) |
| `probe_07_route` | 1 FAILED — the round-4 pre-existing red (the corner leg, item 3) |
| `elevator_buttons_09_live` | 2 passed / 1 FAILED — the round-4 pre-existing red (the cab mouth, census above) |

**The full-suite total line is NOT available this session** — the background `suite.py` dies
between tool calls in this sandbox (round-4's finding, re-confirmed: the started run was dead on
the next tool call), and the foreground window went to the build (twice killed by the 10-minute
tool cap; the rootfs-100% SIGBUS detour above) and the per-binary runs above. The six affected
binaries are covered; the next round should run the fast runner once on this tree.

## What is left

1. Item 1: port the route-stall danim swap (the 0x63 `GoToGoalPosition` blocked arm — the game
   swaps the walk danim for the stall anim so the undrawn raw slide stops with the route), then
   close the shoji red; the clamp consumer facts are in the WORKLOG.
2. Item 3: the corner leg the player's way (live check below first — the two guards' capsules
   overlap across the throat, so the real route cannot pass between them), then re-land the full
   push + mirrored dodge; the elevator_09 blocker needs a trigger census at the stall (NOT a bot).
3. Item 4: armour_0a (the open-panel pocket) and speech_facial (the late guest push breaking the
   seed ray) per collision-4's plan.
4. Run the full four-crate suite on this tree for the total line.

## Live checks for the main agent

1. (stands, collision-4 #4) A fresh-boot 07000007 measurement at the hall NW corner: the two door
   guards' capsules overlap across the throat in the port's census — which way does the REAL
   player's route go past them, and does the real game push Bond there?
2. (stands, collision-4 #1/#2) `gdbwatch` drone+0x4f8 bit 0x1 and drone+0x58c for the driven
   Mayhew at the closed courtyard shoji — the boundary-bit setter and whether the slide stops via
   the clamp or the danim swap.
3. New: 07000009 fresh boot at the cab mouth — what blocks Bond at (-4.76, -368.32, 12.02) →
   (-3.90, -368.60, 11.50) (d=1.00, no world segment, no bot capsule in the port's census): a
   trigger volume or geometry the port's probe misses?
4. (stands, collision-4 #3) The 07000001 contact after Bond's shot near guard 14 — whether the
   real game's HandleExplosives scan ever fires msg 0x21 on that path at all.

## Usage

not visible to me — this session's interface exposes no token, context-window or cost figures.
