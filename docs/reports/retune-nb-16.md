# retune-nb-16 — the 16 chasers-defeat real-damage walks vs the nb-scan-all guard fights

Branch `s/retune-nb-16`, from origin/master 73822d3a0 (unchanged through the session — the final
tree is master + this branch's test change). Task: commit 3412a89a3 made every guard run the game's
spacing scan (`DroneMove_NoBunching` per tick from ControlSTANDARD [M 0x1490f0:202-226], read by
`NDrone2_ChooseCombatMove` as the +0x17 cVar1 [M 0x150c18:58-79]) and the close-range alt attack
from the fire states (`NDrone2_CanAltAttack` first in AimStandFire/AimCrouchFire/SniperFire case 3
[M 0x166328/0x167060/0x15fd88]); the two real-damage chasers-defeat walks of 07000016 die. Re-tune
them. Real input only, no top-ups, never lower damage or health, never change guard code.

## Done

**`walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent` (d2): GREEN, un-ignored.**

- Reproduced the recorded death first (t25168, (-13.23,-22.00,-172.19), byte-identical ledger).
- The fight that kills Bond: with the spacing scan live, `clear_the_pit_shooters` after the chaser
  kill picks NOTHING (no pit-room guard is exposed from the range-keeping fight's end spot), so the
  pit-room AR guard #15 (skin 0x50000b2, weapon 0x14, hp 20) enters the squad phase alive, hunts
  Bond through the 10800-tick stand on the far invisible squad member #13, closes to 2.0 u — inside
  the 3.0 always-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115] — and deals 48.7 in 10
  hits (the run's biggest dealer). The dealer answer between slices fired once (4 of his 20) and
  Bond died INSIDE that stand.
- The lever (one gate change, file-local to walkreal_07000016_live.rs): the squad-phase close
  stands — the 00 Agent law this file already runs (`difficulty >= 3 && !w.tops_up()`, 120-tick
  bounded stands on any live hostile inside 10 u between the member slices) — extended to
  `difficulty >= 2 && !w.tops_up()`. A player answers the muzzle that closes on him.
- Measured green: the close stands answer #15/#14 from the first slices; the squad-phase audit
  passes (21 hits, every delta within the game's chain [D record 99 +0x0C = 50.0 punch; M
  0x1d4424/0x1d3a80's Agent row]); ch 0xbf → 0xc0 → 0x82, the cinema, the button press, ch 100,
  MissionState 6.
- Byte-identical elsewhere: the gate adds `!w.tops_up()` (the tops-up sisters keep their streams
  exactly) and the difficulty bump cannot touch d1 (the Operative walk) or the punch test (d1).

**`walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent` (d3): still red, the
measured ledger and the next lever are in the test's ignore message.**

The walk now reaches the chaser stage in the blast room (was: dead in the gallery pocket at t3093).
Landed levers, all `difficulty >= 3 && !w.tops_up()`, file-local (no crates/*/src changes, no
common/ changes):

1. The east-high ring guards join the peek cycles' fight list (x > 12, y > 5.5, z < -78, with a
   3-fruitless-peek bail so the loop can never burn its budget hopping at an unseen guard): the
   scan strafes a ring guard to (14.1, 6.4, -79.8) whose line the west-only alive filter never
   counted; he dealt 59.5 of the death run's 100 (the biggest dealer by far).
2. The pocket's close pair fights from the peek spot (the pocket stand measured `exposed false` on
   the scan-shifted pair — the whole fight no-oped from the pocket).
3. The exit legs stay FOUGHT (return fire on): crossing unfought measured WORSE — the east-lane
   hunter #1 then deals 75.5 in 22 hits across the open crossing (the 3081 ledger).

Furthest measured point (the ignore's ledger): pocket entry 82.5 → peeks 58.0 → exit fights 19.5
(the lane south kills the ring brute #5 for 0) → doors 19.5 → hand-back 19.5 → room soldiers 9.5 →
dead at t12872, (-14.11,-21.91,-171.89): the chaser's stars + the pit's live wire ("the world"
hit -7.0 at the death tick). The binding constraint measured: AMMO (clip 1 spare 6 at the ramp
foot) and the route bleed.

Measured-worse levers, tried on their own real runs and REVERTED (each is named in the ignore with
its death tick so the next session does not retry them):

- `level_fights` at d3 (the d1 law): skips the elevated fights, #5 survives, his ramp burst kills
  at t3912 (vs t12872 without it — the elevated fights' cost lands while Bond is still 100→58).
- The after-exit resupply: the fetch legs walk back under #5's overhead line (5 hits × 7.0 during
  them), death t3608 (earlier than t3912).
- The ramp-foot perch stand: #5 is exposed TRUE from the ramp foot at 12.3 u, but the gun is at
  clip 1 spare 6 — an hp-20 guard outlasts it, death t4230 inside the stand.
- east-high x > 8.5 + peek arrival r 0.9: the fists classes converge into the pocket (a 0x50000b2
  fists guard punches for 14.0 at d 2.0), death t6008 in the pocket.

NEXT LEVER (in the ignore): the ammo chain — the exit-fight kills drop their own clips
[M DroneWeap_DropWeapon 0x17a2a0] 2-6 u off the exit path, but the resupply's walked-path corridor
missed them (39 trail points); a targeted corpse-hop (the pit loot-pass idiom, walk_nav_to the
corpse spots) at the exit's end refills the 0x1a before the room soldiers, and the chaser stage
needs to hold the shelter-side floor (the wire at (-14.11,-171.89)).

## Verification

- `walkreal_07000016_live` WHOLE at the final tree: **18 passed, 0 failed, 1 ignored** (the d3
  real-damage walk, ledgered). The binary contains the d1 real-damage walk (green), the d2
  real-damage walk (green, un-ignored), both tops-up sisters (green, streams untouched), the punch
  pin (green), the door/interactable tests (green).
- No changes outside `crates/nf-view/tests/walkreal_07000016_live.rs` and the two docs — so no
  other walkreal binary's stream can change (lesson 7's every-walkreal run does not apply: the
  shared helpers under `crates/nf-view/tests/common/` are untouched).
- Every fact above is tagged [M]/[D]/[L]/[S] in the test comments; the death ticks and ledgers are
  [L] fresh runs of the port's own sim on this branch (each lever judged on its own real run).

## What's left

- The d3 walk (above): one session of the ammo-chain lever + the chaser-stage wire handling, then
  un-ignore.

## The fast suite

`tools/test/suite.py` was started twice per the task (once at 19:01, again 19:09 after an ALSA link
fix). Both runs died in the build phase with ENOSPC: the sandbox has 9.9 GB total; the nf-view test
binary set alone passed 5.6 GB when the build was killed at 3.7 GB free, with hundreds of binaries
still to link (the previous session's report documents the same wall). The mini-suite fallback ran
every already-built test binary of nf-game, nf-formats and nf-audio (85 binaries, --test-threads=1)
on the final tree; its total line is below. The task-touched crate (nf-view) is covered by the whole
walkreal_07000016_live binary above; the shared helpers are untouched, so no other nf-view binary's
stream can change.

Mini-suite total: **251 passed, 0 failed, 2 ignored** across the 85 nf-game/nf-formats/nf-audio
binaries (the runner log: `TOTAL (nf-game+nf-formats+nf-audio): 251 passed, 0 failed, 2 ignored
across the built live binaries (0 failing binaries)`).

## Usage

not visible to me
