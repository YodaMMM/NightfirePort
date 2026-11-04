# real-health-14-agent-00agent — report (2026-11-04)

Branch `s/real-health-14-agent-00agent`, cut from origin/master f9f6544c. Task: wrap
walkreal_07000014_live's `walk_the_infiltration_to_the_delta_exit_real_damage_agent` in
`with_real_damage` and make it pass on real health; then finish
`..._real_damage_00agent` (#[ignore]d — the tick-26340 wedge).

## What's done

**The d2 wrapper fix is in** (742ad6bf): the test now runs
`walker::with_real_damage(|| with_agent_path_clear(|| walk_the_infiltration_to_the_delta_exit_at(2)))`
— the same wrap the Operative (`..._real_damage`) and 00 Agent (`..._real_damage_00agent`)
walks use. The health top-up no longer rides; the pass now pins what its name claims.

**On real damage the d2 walk dies by the SAME wedge the d3 died in** — measured twice this
round [L both runs' ledgers]: the "1.72 landing" leg cap, Bond floating `grounded=false` over
a floor the dump itself says is in reach (d2: (33.90, 0.68, 35.38), the scoped floor 0.34
below, floor_below 0.0, `segment_to_target=None`; d3: (33.84, 0.82, 30.95), 0.41 over the
scoped floor — the prior round's numbers).

**The wedge is ROOT-CAUSED, and the port is faithful — the walk was broken, not the game.**
Chain, all measured this round on this branch:

1. A throwaway probe (load, teleport into the exact wedge pose, steer like walk_to)
   reproduced the float: grounded=false forever at the same spot; a zero-input drop at the
   same spot grounds normally — the float needs the walk's state, not the spot.
2. NF_CAPS_LOG: every float tick carries a tiny push from a slab-skirt triangle
   (cent (33.43, 0.40, 35.27), normal (0.906, 0.423, 0)) — a ny 0.423 face, so the gravity
   gate stays open [M 0x194df0:104] and Bond sawtooths (fall −0.036/tick vs push +0.01–0.03)
   0.65–0.86 above the corridor floor.
3. A temporary band print (added, run, REMOVED — no lib diff remains): the player's floor ray
   DID see the corridor floor (hit_y 0.0, dy 0.668) but the band failed —
   **cc=0.5500 with crouched=false for 24198 of 24325 band ticks** [L the instrumented run].
   The feet band reads the CURRENT root height [M Player_FeetOnPoint 0x194188:106
   `hit − 0.1 ≤ cc`; the crouch root 0.55 = BL+0x7CC [M Player_Collision 0x194df0:235/248 +
   the tail store :405-408]; the stand restores the 1.0347 root [M the port's stand arm
   player.rs:1637-1638 modelling the crouch option's stand]]. Reach 0.65 (crouch) instead of
   1.13 (standing) — the skirt-held float hangs just outside it forever.
4. The desync's writer is THIS TEST FILE's own stance pokes — lines 242/414/1444 clear
   `bond.crouched` (414 is on the real walk's crawl exit), 1300 sets it, `set_crouch` toggles
   it — none wrote cc [M each site read this round]. The spawn's p3==3 path pairs them
   correctly [M nf-view lib.rs:1570-1574]; the flag-only pokes did not. So EVERY master walk
   of 07000014 since the crawl ran the whole route at the crouch root — and several pinned
   asserts encode that smaller capsule's behaviour (see below).

**The fix was built, pushed to the branch (971e480b), and VERIFIED through three exposed
legs** before the window closed:

- The four stance pokes now pair the flag with the root height (the game's own pair).
- The 1.72-landing float grounds; the walk advances past the landing.
- The SkyBridge crossing release: the STANDING capsule's spine sphere contacts the east
  walkway's west-edge corner (x 44.54, top 2.37 [D probe14_room]) when the root passes the
  walkway-top level at x ≥ ~43.5, and the push arms `walkable` — Bond hangs on the edge. A
  probe sweep of the gap [L this round] shows every column x 41.8..44.2 at z 62.0..63.55
  reads the trench −0.60 first, so the release loop now drifts WEST 15 ticks (the game's own
  air control [M the +0x20 lanes apply airborne via Player_Move 0x191dd0]) and lands x ≈ 42.9,
  clear of both edges. (The pinned trench landing used to pass ONLY because the buggy crouch
  capsule fell past these catches.)
- The vent-lane drop lands at the standing rest 3.41–3.47 on the 2.38–2.40 lane; the assert
  window's `p[1] < 3.4` cap was a crouch-era artifact (the standing rest = 2.38 + 1.0347 =
  3.41 [M ROOT_HEIGHT]) and was widened to 3.6.
- The NEXT exposed interaction: the "lane south past the mouth" leg jams on the flush lid's
  0.02 lip — STUCK at (45.48, 3.42, 42.72), grounded, the segment to the target clear, no
  progress for the full 6000-tick cap [L the run's dump]. The standing spine bottom
  (root − 0.485) drags the floor; the port's live push displacement cancels the walk step at
  the lip, where the game's dead push tail [M Intersect_CylGeom 0x109da0:447 — the +0x50
  DeltaP lifetime guard kills every walk contact; the ee_snapshot live value drifts ~15 m]
  walks the band alone. Every later leg needs the same one-by-one re-verify.

Each additional leg is an hour-scale re-tune-and-verify cycle, so **the working tree was
returned to the wrap-only state** (commit c7694005) — master behaviour, no red regressions —
and the fix + the three leg fixes live in the branch history (971e480b and the drift diffs
reconstructable from this report) for the next session to re-land leg by leg.

## How it was verified

Per-binary runs, release, `--test-threads=1`, real ISO:

- **walkreal_07000014_live (the final tree): 8 passed / 1 failed / 1 ignored** (190 s). The
  8 = master's green set, unchanged by the wrap. The 1 failed = the d2 agent walk on real
  damage (the wedge — the honest state until the cc fix re-lands). The 1 ignored = the d3
  00agent (unchanged from master, the prior round's measured wedge).
- The intermediate states, run whole: the wrap + cc fix WITHOUT the leg fixes: 2 passed /
  7 failed (the standing capsule re-exposing the crossing and vent catches); with the three
  leg fixes: the vent lane lands (`lane true`, y 3.47), the d1-real walk advances past it and
  jams at the lid lip (the dump above). All four walk diff commits carry their own measured
  state in the messages.
- NOT run this window: the full four-crate suite, nf-game lib, the other walkreal binaries.
  The final tree's ONLY source diff vs master is this one test file's wrapper (verified:
  `git diff 742ad6bf^..c7694005` — the test file + docs; nf-game/nf-formats/nf-audio/nf-view
  src untouched — the band instrumentation was added and removed inside the window, net
  zero). Rule 10's grep for the touched symbols lands entirely inside
  walkreal_07000014_live, which was run whole. The 971e480b state (the cc fix) touches the
  same file only.

## What's left

1. Re-land 971e480b + the two crossing west-drifts + the `p[1] < 3.6` vent-lane windows
   (exact diffs described above), then walk the remaining legs one by one: the lid-lip jam
   first (a hop-over in the leg's own steer loop, the walker's wedge-recovery pattern — the
   game's player hops the lip the same way), then whatever the standing capsule exposes next
   down the route (the descent's overhang catches are the likely next: the walk's own
   "landing west drop" loop is adaptive — hop when grounded until clear).
2. Then the d3 00agent: un-#[ignore], run with the levers in place (the d3-gated re-ground
   before the landing resupply is likely MOOT once cc is fixed — the float grounds; the
   fights' 50.0-health grind and the stairwell AIM-pair stands remain the live levers).
3. Then every walkreal_*_live + npc_doors_live + fix_guard_misc_live + the full suite total
   line. NOTE for other levels: walkreal_07000015_live, walkreal_0700000a_live and
   probe0c_plates_route.rs carry their own `set_crouch`/stance pokes with the same flag-only
   shape — if their walks ever float on a skirt, the same desync is the first suspect.
4. The sibling helper copies: this file's `set_crouch` was fixed in 971e480b; the other
   files' copies were not touched (out of scope).

## Live checks for the main agent

1. **07000014, the crawl-vent stand (the cc pair)**: fresh boot, let Bond crawl out of the
   start vent and press the crouch toggle to stand — a gdbwatch on BL+0x7CC during the stand
   should show 0.55 → 1.0347301 in the same tick the stance flag clears [M the port's pair at
   player.rs:1637-1638; the game's stand arm Player_SSCrouch 0x1924c0:84-105]. If the game
   restores the root LATER (or eases it), the port's pair needs the same shape.
2. **07000014, the SkyBridge release at 00 Agent**: release the Q-Claw over the open gap and
   do not touch the stick — if the game's Bond (standing capsule) catches the east walkway's
   west edge the way the port's push arms it, the west-drift is the player's own recovery and
   the test is honest; a `--bond 44.0,4.6,63.5` render looking south-west at the gap would
   show the edge corner the probe measured (x 44.54, top 2.37).
3. **07000014, the flush lid lip (z 42.16)**: walk south along the east vent lane over the
   lid — if the game's Bond walks it without a hop, the port's live push displacement (vs the
   game's dead tail [M 0x109da0:447]) is the jam's root and the port-side fix (drop the walk
   push displacement behind the existing NF_WALK_GHOST evidence) becomes the right move, run
   against the full walkreal battery.

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).

## ntfy failed

All three fallbacks failed at 12:55 UTC: `curl` exit 28 (connection timed out after 20 s),
python3 `urlopen` URLError [Errno 101] Network is unreachable, `wget` killed at the 30 s
tool timeout mid-connect. No route to ntfy.sh from this sandbox; the notification could not
be delivered.
