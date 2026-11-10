# los-mask-repins-08 — report

Branch `s/los-mask-repins-08` = `origin/master` c120d7b49 (the exact guard sight on:
823ff190b per-object masks, cel walk, sight rotation, guard's own cel, own-cel boundary
push, FellOutMap). Task: bring the 07000008
`walk_the_whole_mission_on_real_input_real_damage` / `_agent` walks green under that sight
and remove their ignores.

## What the ledgered deaths actually are

The ignores (from the los-mask-repairs ledger) said "green with the flat feet-height
boundary push". That was the pre-823ff190b guard push: the round walked EVERY boundary wall
from the feet. Round 3 of los-mask-repairs ported the game's own-cel push
(`NDrone2_Collision` 0x140e90:169-174 → `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0
walks only the guard's cel +0x34 wall list) because that is what the decompile says the
game does — 01 needs it. So the 08 walks had to survive the exact push, not the old one.

The death is NOT a guard-sight fight move. NF_DMG_TRACE on the Operative walk
(deterministic, reproduced tick-for-tick at the ledgered 29332): Bond stands inside the
bottom terminal at (111.1, 13.5, −179.7) after the ride, shooting the Hind down, and the
"world" ledger bucket is the Hind's OWN strafing rounds — rockets detonating on the
carriage's own wall band next to the standing spot, −1.20 × 15 (frames 29154-29249), then
the −23.00 killing blast at 29332. The machine's rocket lines fly at Bond's launch-time
position; the walk stands the strict launch gates still, so every detonation lands on the
band beside him. The guards' sight change moved nothing here; the fight just got honest.

## The re-route (real input only)

The missile dodge the 00 Agent walk already carries (the laser-hinges-2 side step off each
inbound round's line, real strafe input, hold-fire while moving, the launch gates re-read on
the settled spot) was D3-gated so the sisters' timelines stayed byte-exact. This task
un-gates it for the real-damage walks with the SAME gate shape the sight gates already took
(the los-mask-join extension): `difficulty >= 2 || !tops_up()` — the topped-up Operative
walk keeps its byte-exact timeline. No damage or health writes, no top-ups; the helper
lives in the 07000008 test file and the gate is by difficulty, as required.

## Result

- `walk_the_whole_mission_on_real_input_real_damage` UN-IGNORED, GREEN: end health 1.2
  armour 0.0, 116 hits for 98.8, 17 return-fire fights, the Hind down (damage 100.6 on the
  machine at f29310, dead by f31140), the whole mission walked on real input in 31146 ticks
  with both objectives folded and the exit hand-off — the run the un-ignoring commit cites.
- The Agent sister stays IGNORED with a fresh ledger: the dodge answers the ledgered Hind
  fight (Bond reaches it and moves off the rocket lines), but the Agent run bleeds out at
  tick 37022 — the −1.40 × 34 machine blasts through the shot-out panes over the pre-fight
  ledger (burst-pair #19's 33.6 at the lower level east, Agent accuracy 1.1
  [M DroneWeap_DoBulletAccuracy 0x179740:74-88]); 69 hits for 100.0. Its ignore message now
  carries the traced ledger and three levers (fight from the west-doorway slit; open the
  unseen-cover fights to d2; widen the dodge to the detonation band).
- The 00 Agent sister keeps its earlier ledger (its own levers, unchanged).

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live | 4 passed / 0 failed / 2 ignored (92.9 s) |
| interactables_07000008_live | 15 passed / 0 failed (16.0 s) |

Only `crates/nf-view/tests/walkreal_07000008_live.rs` changed; nothing under
`crates/*/src/` and nothing under `tests/common/` was touched, so no other binary's
behaviour can move and the shared-helper opt-in rule is not triggered.

## What did not run

The full four-crate suite did not fit this sandbox: 9.9 GB of disk against a single-set
release build of nf-game/nf-formats/nf-audio/nf-view plus its ~100 test binaries (a
mid-build RUSTFLAGS addition also forced one cargo clean + rebuild), and background jobs
die between tool calls here so the fast runner (`tools/test/suite.py`) could not stay up.
The two binaries above are the task's own requirement; a full-suite pass on the merged
final tree is the first thing the next window should run.

## Unproven / for the next window

- The Agent levers above are suggestions ([S] with their measured anchors), not pinned.
- Whether the −1.20/−1.40 per-detonation falloff at the carriage matches the game's
  Explode_Propagate band next to a wall is not compared against a live run; the signatures
  (small repeated hits, then one big wall-band blast) are the d3 ledger's [M
  Explode_Propagate 0x17c518:69-71].
- Live checks for the main agent: none required — the walk is deterministic and its ledger
  is traced; a PCSX2 look at the Hind fight (does the game's Bond stand in the terminal and
  eat band detonations, or fight from the doorway lip?) would settle the Agent lever (2)
  faster than sim bisection.

## Usage

not visible to me — this sandbox shows no token, credit or context-window figures.

## ntfy failed

All three fallbacks failed from this sandbox (egress to ntfy.sh blocked):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "los-mask-repins-08: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 urllib.request` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "los-mask-repins-08: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, timed out after 30 s (killed).

The queue entry (done/30-los-mask-repins-08.md, pushed with the report attached) is the
notification of record.
