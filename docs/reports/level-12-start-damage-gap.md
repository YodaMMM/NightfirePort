# level-12-start-damage-gap — the 07000012 idle start still gives back less than the game

## What the task asked

`the_idle_start_costs_bond_what_the_games_does` (ignored on stream batch 1121d): the port reads
82.50 health at tick 480 and 28.25 at 840 on this branch's base (the ignore message's 80.75/56.25
was the pre-batch-3 tree) against the game's 53-72 / 0-37 [L]. Find which gate or timer differs
from the decompile (NewSighting, ChooseCombatMove 0x150c18, the stand/crouch swap hold), fix it
with citations, un-ignore the test.

## What this round found (all measured on real input through `load_with_carry_difficulty(.., 1)`, idle Bond)

- **The 840 side of the assert now passes on master** (28.25 <= 40.0; the game's band 0-37 [L]).
  The whole remaining gap is the first 480 ticks: the port loses 17.5, the game 28-47 [L].
- The port's chain, probed tick by tick: the pair alerts at t169 (the level script's sound 634 at
  the south guard's spot), los t170/171, the ack at 10 sighted ticks (reaction time 10 [M
  `DroneFunc_ReactionTime` 0x14ac60 default arm]), fire states t179/180, first shots t180/181.
  33 rounds by t480; 21 real rolls after the 120-tick NewSighting gate opens (~t290; [M
  `DroneWeap_DoBulletAccuracy` 0x179740:103-108 on +0x270, and the ELF static 2.0 x 60
  [D ACTION.ELF 0x30cafc] match the port's tuning). 9 rolled hits, 6 landed.
- **3 rolled-hit rounds die on the counter edge** at (17.4-18.4, 1.3-2.0, ~19.0) between the
  guards and Bond (World events, no Player event). The crouched rounds (muzzle y 0.84) clear
  under it — the south guard's landed hits all came from 0x70.
- **The captain assignment is not the bug.** [D] placement plc 13 (south) carries prop 9 (the d1
  captain bit [M `NDrone2_DefaultInit` 0x14b300:504-508]), so the port's south guard is the
  3.5-per-hit shooter exactly as the disc says, with the swap in Attack 0x56 [M
  `NDrone2_DSTATE_Attack` 0x164210:31-52]. My first naive bit decode of the placement words said
  prop 0x40 was set on both guards — wrong: the game maps properties through bitDescs/bitMasks
  [M 0x1c7d98], and through that table both guards carry 0x40 CLEAR, prop 0x11 set, and the south
  guard alone carries prop 0x13.
- **Landed fix ([M] 0x166328:30-49):** the port's `AimStandFire` ENTER now carries the game's
  prop-0x40 / movement-locked diversion to CombatNoMove 0x59, and the captain's 0x19->0x18 gun
  swap (AimStandFire's own copy; AimStand 0x67 ENTER 0x166178:33-42 already had it in the port).
  Both are no-ops on this level's pair, but they are real decompile arms the port lacked.
- **Verified divergences recorded for the next round, not landed:**
  1. The port's north guard sits in AimStandFire 0x68 from t179 to t840+ with ZERO
     `ChooseCombatMove` re-picks, while the game's north guard hits from 0x70/0x74/0x68/0x6f [L,
     prior task's PCSX2 per-hit log] and ends at (13.9, 17.1-17.5) [L] — repositioned, so its
     standing sight line to Bond clears the counter edge; the port's guard never moves and its
     standing rounds die on that edge. The port's between-burst `combat_state` ask window is the
     ~1-tick `burst_done` drain tick (the weapon pass re-clears it next pass), and the probe
     caught the aim danim ending into class 7 between bursts (t179-260), which refuses
     `Ready2Fire` [M 0x178c48:47-54] until the re-call flips it back to class 2.
  2. The game's per-round guard spread is weapon+0x24 x Drone+0xc0 where +0xc0 = the accuracy
     byte as a float [M `Bullet_init` 0x12c5a0:139-148 + `NDrone2_DoModeSettingsNEW`
     0x14c600:108; also `BOT_setDroneStats` 0x126318 on the MP side]. The port passes the raw
     +0x24 (10.0 for 0x18/0x19 [D]) — about 7x too precise for acc 7. This is a real divergence;
     it makes the port's rounds *more* accurate than the game's, so it is not the missing-damage
     cause, but it must be fixed for the spray/wobble dynamics to match.
  3. The port's step-aim states (0x73/0x74) hold fire until the next 0x68 entry, while the game
     logs hits from 0x74 [L]: the game's 0x74 ENTER clears +0x3b once [M 0x1676a0:24] and the
     weapon pass re-stamps it; the port's `firing = false` latch blocks longer.

## What's left

- Instrument the ask window (+0x3c vs the port's `burst_done`) and the anim class-7 window, make
  the port's 0x68 between-burst ask rate match the game's case-3 shape [M 0x166328:121-151], and
  re-measure the 480 band.
- Port the Drone+0xc0 spread scale into `fire_weapon`.
- Judge the port's 0x74 hold-fire latch against the game's weapon-pass re-stamp.
- Un-ignore the test when `at(480) <= 72.0` holds.

## How this was verified

- `start_rounds_07000012_live` (log_every_round...): ok — 33 rounds, 6 landed, 17.5 lost at 480.
- `start_pair_07000012_live`: the_start_pair_keeps_apart... ok; the target test still ignored
  (the refreshed ledger is in its ignore message).
- `walkreal_07000012_live`: 4/4 ok after the ENTER change.
- `cargo check --offline --release -p nf-game`: clean (only pre-existing warnings).
- **The full suite did NOT fit this session** (setup + probes consumed the window; the sandbox
  expires 07:12 UTC). The only code change is the AIM_STAND_FIRE ENTER arm; the next round should
  run the full suite before further pushes.
- Scratch probe kept at `crates/nf-view/examples/probe12_sight.rs` (NF_ISO opt-in) — the sight
  chain / round-and-impact / ask-gate variants are in its git history.

## Live checks for the main agent

1. Guard 0x1877240 (north) and 0x1876400 (south) from a fresh boot of 07000012 d1, idle Bond:
   log `Drone+0x10c` (state), `+0x270`, `+0x274`, `+0x3b`, `+0x3c`, `+0x568` (the playing danim's
   class), `+0x56a` (its anim id) and `+0x100` every tick 150-500. Expect: which states the north
   guard actually occupies (the port has it pinned in 0x68; the prior task's log claims
   0x70/0x74/0x68/0x6f hits), when +0x270 starts accumulating (the game's alert tick), and the
   danim class during 0x68 (2 vs 7 — the port's between-burst class flips to 7).
2. Each guard's position every 10 ticks 150-480 (x,z only) — the north guard's drift toward
   (13.9, 17.1-17.5) [L] vs the port's static (13.2, 18.55-18.8).
3. A `Bullet_init` watch for owner 0x1877240 in 0x68: the origin, dir, and the spread draw — to
   confirm the +0xc0 spread scale lands in the game's rounds (the port computes ±0.014 rad).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this environment).

## ntfy failed

All three fallbacks, exact output:

1. curl: `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: `Connecting to ntfy.sh (ntfy.sh)|129.212.146.227|:443... failed: Connection timed out.` /
   `Connecting to ntfy.sh (ntfy.sh)|2604:a880:800:14:0:3:cb8d:4000|:443... failed: Network is unreachable.`
   (plus three more IPv6 `Network is unreachable` lines).
