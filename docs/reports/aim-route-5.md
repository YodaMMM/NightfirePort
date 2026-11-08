# aim-route-5 — the drakes walks green on the snap stack too (both aim-route-4 §4 reds resolved); 0c's far-sniper cause pinned to the still-gate; 08/16 unchanged

Branch `s/aim-route-5` from `origin/s/aim-route-4` (b991dc45d), merged `origin/master`
(ab80ac6ed — the one-way doors + the escort VIP fail; **no `crates/*/src` file changed in the
merge** — tests and docs only). Two commits after the merge: the walkreal port + the
worklog/report. **Test-side only**; the [L] 08 pins and the 0x30000000 fix untouched (no game
code changed on this branch).

## 1. The drakes walks (aim-route-4 §4) — both green

The §4 reds were (1) the plain walk's Drake fight (Drake alive at the 6000-tick cap) and (2)
the real-health walk's console starvation on 7/1/2 (the run died at 50 hits for 100.0). Both
resolved by porting the master raw-aim fix (missile-countdown-master cf0b7357e) into the
recipe file, adapted:

- The console tail serves the SHARED +9599 deadline as **[1, 2, 7]** (equal deadlines — free
  order); the wait spends at 1/2's x20 station, 25 units from the wave's nest.
- **The wave defense, TAIL ONLY**: whenever the astronaut wave closes inside 25 u on a tail
  console, the walk fights it with the proven first-wave killer (the fight's dry-clip cover
  discipline paces the 100-round pool against the 12-a-tap drain [M 0x193eb8-0x19405c]).
  TAIL ONLY is load-bearing: master measured an unrestricted detour at +829 on k3 — m3 then
  read its console CLEAR at its +4199 one-tick branch and took the LAUNCH arm
  (MissileDeploy [2,2,3,…], mission fail).
- The death block waits bounded (550 < the NIS's t600 [D 0x06000a20]) for the monitor's
  ch-0x09 record read [M 0x1827d8:78-85] — on the disc clocks m1's p2 end (disarm+1710 [D
  0600091c], p0 3300 [D P0_LEN]) lands ~450 ticks after the fight gate.

**Why Drake's fight now kills (the §4 cause, settled)**: the fight code is identical on both
branches; the difference was the WAVE. In the old shape the wave was alive at the fight
(waited out, not killed), and the fight's own target pick ranks "an astronaut inside the 3.5 u
always-hit band" above the prio target — the floaters kept stealing the taps and grinding the
stand (§4's "float-fight aim pick" suspect, confirmed). With the wave answered mid-phase the
fight faces Drake alone and he dies well inside the cap — on the raw aim AND under the snap
stack. The clip state was never the blocker.

Measured latches (plain walk): 5@3993 (−553) 4@5217 (−529) 3@6417 (−529) 6@7593 (−1153)
8@10438 (+1692) 1@10743 (−2995) 2@10851 (−2887) 7@11553 (−2185); Drake@13606 dead; deploy
all-2; ch 0x09/0x0b/100; mission 6; both objectives 4. The real-health walk survives the whole
route on the same shape.

## 2. Verification — every binary whole, per-binary counts

| binary | result |
| --- | --- |
| `walkreal_0700001b_live` (whole) | **17 passed / 0 failed / 2 ignored** — both drakes walks green |
| `drake_fight_live` (whole) | **3 / 0** |
| `interactables_0700001b_live` (whole) | **9 / 0** |
| `space_laser_1b_live` (whole) | **4 / 0** |
| `nf-game` `--lib` (whole) | **686 / 0 / 5 ignored** (space_missile 6/6 inside) |
| `walkreal_07000008_live` (whole) | **5 / 1** — the same single pre-existing red (`…_real_damage_00agent`), unchanged by the merge |
| `walkreal_07000016_live` (whole) | **13 / 1 / 2** — the same single pre-existing red (chasers real_damage), unchanged |

- The full 4-crate suite did NOT run (time). **No `crates/*/src` file changed on this branch**
  (the merge brought tests/docs only; the walkreal change is one test file), and every binary
  whose level the branch touches ran whole, listed above.
- Drake's fight: **GREEN** on the aim stack — the aim-route-4 §4 red is resolved with its cause
  (the wave stealing the fight's 3.5-u band picks), not by re-pacing.

## 3. 0c — reproduced; the cause pinned one level deeper than §3; levers for the next window

`walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage` re-ran red at the SAME
tick (13461, 102 hits for 100.0, 9 return-fire fights; the death ledger's finishers #39 d 61.3
and #20 d 64.3, one 16.0/11.0 hit each, both at full hp 10).

The driver's weapon check is DONE and CLEARS the weapon: Bond holds the 0x1e record itself
(the boot loadout's switch request 30 [L the 0c boot log]); 0x1e's +0x1C is 1200 — the sniper
family [D weapon_data 0x2bf150; the port's bullet-range-210 unit test fires a 0x1e record past
150]. Range is NOT the blocker.

Two levers were tried and REVERTED (both measured the identical 13461 death):
1. The shooters window opened to (95, 13) (`w.dmg.shooters_window`, the existing per-test
   opt-in the 05 walk uses) — the fights then RAN on the far snipers (NF_RF_TRACE: converged
   err 0.000 / perr 0.000, blocked false, clip draining) and still landed NOTHING.
2. `with_real_damage_unseen_fights` (the hurting-first pick + the creep) on top — same death.

**The cause**: the return-fire stands' converged AIMED rounds miss a guard that STEPS during
the round's flight — the round flies [M Bullet_update 0x12e7b0] and a wiggle-step of ±0.1 u at
d 60-90 is enough; the level's own file documents the identical mechanism for the deck sentry
("converged aimed rounds … never landed while it wiggle-stepped … the same stand LANDED the
run" — walkreal0c.rs's still-gate note) and solves it with a STILL GATE: tap only while the
guard is not stepping. The return-fire path (`stand_and_fire`) has no still gate. The snipers'
hp 10 also means ONE landed round kills — every gate after the first hit is trivial.
**Next-window lever**: an opt-in still gate in `stand_and_fire` (set by the 0c walk only), or
a 0c-file-local pre-clear of #39/#20 with the file's own still-gate stand fn.

## 4. What's left

1. 0c per §3's still-gate lever (the window + the unseen fights alone are measured no-ops).
2. 08 `…_real_damage_00agent` (the station-legs hurt volume, §3's classification stands) and
   16's chasers real_damage (the melee swarm) — unchanged reds, their levers in
   1b-zero-g-floor §3; not attempted this window (the drakes walks consumed the priority).
3. 07000014's three nav reds (the doorway waypoint, §3) — untouched, not this task's list.

## Live checks

1. **The m8 timeline gap (carried from missile-countdown-master)**: both branches now measure
   k8 latching ~1700 ticks PAST the P0_LEN model's +5999 branch read (aim: 8@10438 margin
   +1692; master: 8@10463) with m8 DISARMING — the model's read tick for m8 runs ~1400-1700
   ticks late. Watch the kind-0x39 missile at placement (51.78,−12.84,+11.38) in PCSX2 with
   ch 193/194 held: the frame its obj+0xf4 leaves state 2, relative to ch 8's rise and to
   launch+3600+2399.
2. **The 0c sniper still-gate confirm**: with the debugger, read one 0x1e round's flight while
   #39 wiggle-steps at d 61 — expect the round's closest approach to the chest box to miss by
   the step's lead; and again with #39 still — expect the hit. That pins the still-gate lever
   before the next window builds it.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
