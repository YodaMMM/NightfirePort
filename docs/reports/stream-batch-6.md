# stream-batch-6 — the joined tree: bisect, causes, the battery

Branch `s/stream-batch-6` from `origin/s/stream-joined` (2efbce7a3). Task: the failing count
doubled from 28 (stream-batch-3) to 59 — bisect the new reds first, then causes, then the
walks. Time-boxed window; nothing here changes game code or loosens a pin.

## 1. The bisect (7 named binaries, `--test-threads=1`, real ISO, whole runs)

| commit | ai_live | anim_events | guard_barks | guard_reaction | guard_side_gates | spawner_kill | tase_silent |
|---|---|---|---|---|---|---|---|
| stream-batch-3 659429d32 | 37 ok | 5 ok | 4 ok | 1 ok | 4 ok | 9 ok | 2 ok |
| spawn-pick-and-dart-stamps base 29c667a30 | ok | ok | ok | ok | ok | ok | ok |
| **c4b2d38ce** (the spawner pick draw) | ok | ok | ok | ok | ok | **2/7 red** | ok |
| **f1f37ba47** (the spawn-chain draws + the dart stamp block) | 36/1 red | 4/1 red | 3/1 red | 0/1 red | 1/3 red | 2/7 red | 0/2 red |
| batch-4 tip 95772586e / batch-5 pre-AI 15e02952c / the joined tip | red | red | red | red | red | red | red |

Verdict, by commit:

- **spawner_kill's 7 reds = `c4b2d38ce`** ("the spawner pick draws Rand_Rand over the WHOLE
  template list and a rank past the usable count spawns nothing"). Verified game-shaped this
  session against the decompile: SpawnDrone 0x137cb0:19 draws `Rand_Rand(*(ushort*)(world+0x2e))`
  (the whole-template count), the rank walk decrements per ELIGIBLE template (:31-56) and a walk
  that exhausts the list spawns nothing (:88-96) [M]. The 7 tests pinned the old
  `rand(usable.len())` always-spawn behaviour (the commit's own fail-first unit tests are in the
  module).
- **The other 10 reds = `f1f37ba47`** (the spawn-chain draws + the dart Tower stamp block).
  Also verified game-shaped against the decompile: DefaultInit 0x14b300:382-461's per-param-14
  draw counts and order (default/1/2/5 two `Rand_FRand`s, 3 two `Rand_FRand(6.0)`s with
  `+0xFC = 100 + second`, 4 one `Rand_FRand(8.0)`, then the unconditional `Rand_FRand(0)` →
  +0xF4 tail at :435), and the wobble stamp `Rand_Rand(10000) → obj+0xEC` AFTER CreateObj at
  SpawnDrone 0x137cb0:58-59 [M]. The port's new draws match count-for-count and order-for-order.
- **116695f5a (batch-5's re-acquire exit removal + the scoped sight ray) is CLEARED**: the same
  16 reds reproduce at 15e02952c (batch-5 BEFORE that commit) — they predate it.

**Why the tests are red over game-correct code:** every red test pinned the port's OLD
spawn-draw-free RNG stream. The two commits add the game's own spawn-path draws, which shifts
the shared stream for every level with spawner waves — the guards' aim/step/bark/cooldown rolls
move (ai_live's SG5 gap is 136 not 4 because the guard's step roll now lands a step between the
first bursts; the tase guards' roll sequence differs; the reaction-floor draws differ). These
pins cite live traces (live 61/101/113, the 0x39 routes), so re-pinning them needs live
re-measurement on the new stream — the [L]-pin rule forbids doing that blind in this sandbox.
**Say-which-commit answer: c4b2d38ce (spawner_kill) and f1f37ba47 (the other ten).**

## 2. The other non-walk reds (causes named; nothing touched)

Already red at stream-batch-3 with documented causes (batch-3 report): goggles_draws
`a_captains_laser_ray` (the press eaten by the start-room stagger), upward_shot_0700000c 0/2
(the 15-draw cone realization moved on the joined streams; needs the file's own PCSX2 re-check),
start_corridor d2 legs (the 0x5c follow-wedge stall — see below), mayhew_hold + fix_npc_doors
(the escort plate/radius wedges). This window adds only the shape of two:

- abseil_elevator `the_rope_follows_the_opponent_down_and_lands`: "hung at the rope top (test
  1)" — 143 vs 144, an off-by-one wedge at the rope top.
- escort_follow_07000001 + fix_npc_doors share ONE shape: the 01 escort NPC settles at
  (3.0, 6.3, 7.3) and never advances to the plate/shoji doors — the same escort-advance wedge
  mayhew_hold's report named (the radii ledger).

## 3. The 0x5c follow wedge — NOT attempted

The d2 stall's next lead stands as batch-5 left it (route.follow reports MOVING while the body
never advances behind the column; the step-back arms [M 0x15a2c8] never fire because follow
never returns BLOCKED). The game's own re-position shape needs the live checks batch-5 listed
(PCSX2 07000011 d2 idle Bond; gdbwatch LinkCreep_Decrement 0x15a148). No game-code change was
made without it.

## 4. The walkreal battery (11 binaries, `--test-threads=1`, real ISO)

| binary | passed/failed/ignored | red legs |
|---|---|---|
| walkreal_07000003_live | 4/2/2 | real_damage, real_damage_agent |
| walkreal_07000004_live | 8/2 | the stats-row and results-page tests (not the walk legs) |
| walkreal_07000006_live | 4/2/1 | real_damage, real_damage_agent (batch-5's pre-existing pair) |
| walkreal_07000008_live | 5/1 | real_damage_00agent |
| walkreal_0700000b_live | 6/1 | real_damage_00agent — **improved from batch-5's 5/2** (the agent leg recovered on the joined tree) |
| walkreal_0700000c_live | 2/1/1 | real_damage |
| walkreal_0700000d_live | 1/1/1 | real_damage |
| walkreal_07000012_live | 2/1 | real_damage_agent |
| walkreal_07000013_live | 1/2 | real_damage, agent |
| walkreal_07000014_live | 6/3/1 | real_damage, agent, + the tripbomb route (pinned 9/0+1i on gas-rng-draws-6; here the placed tripbomb is no longer stuck at the retreat — a fight/flush shift) |
| walkreal_07000015_live | 5/1/1 | real_damage_agent |

Interactables (the last listed binary): interactables_07000001_live 9/1 —
`the_bond_bonus_walk_over_takes_it` hits a walk_to cap: Bond stuck at (−9.52, 7.10, 9.40)
1.40 from the target (−9.50, 5.80, 8.00), grounded, no doors/triggers nearby — the walker
cannot descend the 1.3 floor step to the bonus (the "walk_to cap needs a look at what stands
there" item).

No plain walk regressed against batch-5's battery; every red is a real-damage/Agent/00-Agent leg
(the item-4 lane) except 04's two stats tests and 14's tripbomb.

## What's left

1. The two stale-pin families (16 reds) — re-pin on live re-measurement of the post-draw stream,
   or revert the two game-shaped commits if the driver prefers the old stream (not recommended:
   the draws are the game's [M]).
2. The 0x5c follow wedge after the main agent's two live checks (batch-5 report §3).
3. The item-4 lane: the real-damage/Agent/00-Agent legs above, 04's stats tests, 14's tripbomb
   wedge, escort_follow/abseil/goggles/upward_shot.
4. The full four-crate suite did not fit this window; the bisect binaries, the non-walk set
   and the walkreal battery above ran whole.

## Total

**110 passed / 42 failed / 7 ignored across the 25 binaries this window ran whole
(--test-threads=1, real ISO): the 7 bisect binaries 45/17, the non-walk set 21/8, the 11
walkreals 44/17/7.**

## Usage

not visible to me

## ntfy failed

Fallback 1 `curl -sS -m 20 -H "Title: NightfirePort" -d "stream-batch-6: ..." https://ntfy.sh/nightfire-yoda-8k3x`
→ `curl: (28) Connection timed out after 20002 milliseconds`, exit 28.
Fallback 2 python urllib → `URLError: <urlopen error [Errno 101] Network is unreachable>` (the
same failure the void-recovery-2 window recorded).
Fallback 3 `wget -q -O- --timeout=20 --post-data "stream-batch-6: ..." https://ntfy.sh/nightfire-yoda-8k3x`
→ exit 4 (network failure), no body. All three failed this window.
