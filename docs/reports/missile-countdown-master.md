# missile-countdown-master — the proved missile countdown ported onto master alone; all three 0700001b binaries whole and green on master's raw aim

Branch `s/missile-countdown-master` from `origin/master` (ab80ac6ed). Five commits: three
cherry-picks (the exact commits the task named) + the walkreal fix + the worklog/report.
**No aim snap code, no other aim-stack commit** — the cumulative diff vs master is
`space_missile.rs` (the constant + the seed-order docs) and the three 0700001b test files.

## 1. What shipped

- **085b88ca7** (`s/aim-route-4`), cherry-picked clean: `COUNTDOWN_STEP` 512.0 →
  `REC_FRAME_RATE` = 1/60.0 [M the state-0 body 0x1aa1f0-0x1aa208 loads 0x30d0dc with
  `lui $at,0x31; lwc1 $f1,-0x2f24($at)`; symtab `0030d0dc` = REC_FRAME_RATE; the compare is
  c.olt.s/bc1f 0x1aa200-208] + the seed-index doc map.
- **bec12dff9**, cherry-picked with ONE conflict resolution: the corrected seed order
  (105/105/40/20/0/60/105/60 by missile index [D placement params re-dumped 2026-11-06]),
  `CONSOLE_ORDER`, the 12000-tick phase cap, and the drake_fight/interactables re-pacing.
  The conflict was `walkreal_0700001b_live.rs` only — the aim branch's version carries ~300
  lines of aim-route recipe (0c07b665a..52751fdf7, the aim-snap-stack test commits) that
  master must not take; the resolution keeps master's stall tracking and adds only the
  12000 cap + the CONSOLE_ORDER/doc.
- **8e4094c45**, cherry-picked clean: the drake_fight klaxon drive (the disarm drive shoots
  the held UIs from the first gate; the all-armed pin replaced by the all-disarm pin).
- **The walkreal fix** (own commit, test-side only, this level's file): on master the disc
  countdown left both drakes walks red — the SAME shape the aim branch's report §4 recorded
  (its own walks shipped 15/2):
  1. **The real-health walk died mid-phase** (50 hits for the full 100.0): the station's
     astronaut wave follows Bond across the map (measured: 3.0-damage hits at d 3-13 from
     frame 9414 while he hovered the k1/k2 wait) and the [7,1,2] shape hovered INSIDE the
     wave's nest (x46) for 7's UI (+6659). Fixes: the tail serves the SHARED +9599 deadline
     as **[1, 2, 7]** (equal deadlines — the order is free; the wait spends at the x20
     station, 25 units from the nest), and whenever the wave closes inside 25 u on a tail
     console the walk **fights it with the proven first-wave fight** (its dry-clip cover
     discipline paces the laser's 100-round pool against the 12-a-tap drain [M
     0x193eb8-0x19405c]). **TAIL ONLY**: an unrestricted detour measured +829 on k3 — m3
     then read its console CLEAR at its +4199 one-tick branch and took the LAUNCH arm
     (MissileDeploy [2,2,3,…], mission fail) — the early chains have only ~560-880 margin.
  2. **The ch-0x09 pin mistimed** (plain walk, line 725 red): m1/m2/m7's p0s run 3300 ticks
     [D P0_LEN], so the last p2 end (m1's, disarm+1710 [D 0600091c]) lands ~450 ticks after
     the fight gate — and master's raw-aim fight killed Drake INSIDE that window (the aim
     branch never reached this assert: its snapped Drake never died in the 6000 cap). The
     death block now waits bounded (550 < the NIS's t600 [D 0x06000a20]) for the monitor's
     ch-0x09 read [M 0x1827d8:78-85] and pins the honest order: obj 1 completes strictly
     before the NIS's 0x0b.

## 2. Verification — every binary whole, per-binary counts

| binary | result |
| --- | --- |
| `walkreal_0700001b_live` (whole) | **17 passed / 0 failed / 2 ignored** — both drakes walks green: plain latches 5@3987 4@5145 3@6345 6@7587 8@7864 1@10187 2@10300 7@10810, Drake@13606 dead, deploy all-2, ch 0x09/0x0b/100, mission 6, both objectives 4; real-health latches 5@3987 4@5145 3@6345 6@7587 8@10463 1@10751 2@11116 7@11620, health **3.0** remaining (49 hits for 97.0 — survived), Drake@13606 dead, same end pins |
| `drake_fight_live` (whole) | **3 / 0** (incl. the re-paced klaxon/launch-sequence test) |
| `interactables_0700001b_live` (whole) | **9 / 0** |
| `space_laser_1b_live` (whole) | **4 / 0** (the missiles inert there) |
| `nf-game` `--lib` (whole) | **686 / 0 / 5 ignored** (space_missile unit tests 6/6 inside) |

- The full 4-crate suite did NOT run (time): the branch's only src change is
  `space_missile.rs` (read by the four 1b binaries above, all run whole); one test file
  changed; nothing under `crates/nf-view/tests/common/`; no other level's walk reads the
  missile chain.
- **Drake's fight is GREEN on master** — the raw aim killed Drake inside the 6000 cap (the
  aim branch's Drake-fight red was its own stack's issue; the cause there — the float-fight
  aim pick + clip state after the console phase — does not reproduce on master).

## 3. What's left / notes

1. **The real-health walk survives on 3.0 health** — thin. The wave tanks ~2600 ticks of
   fire during k8's window (the defense is tail-only: k8's own margin is ~880 and the m3
   lesson forbids a detour there). Levers if a later window wants margin: thin the wave
   earlier (the defense radius/order) or a k8-safe cover stance.
2. **The m8 timeline model gap**: the real-health run's ch 8 rose at 10463 (the loop records
   the rise tick; no fight ran in that window) yet m8 disarmed — while P0_LEN[7]=2400 and
   seed 60 put m8's one-tick branch read at ~launch+6240 (~9000 abs). Something in the
   missile timeline runs ~1400 ticks later than the P0_LEN model predicts (see the live
   checks). The tests pin the OUTCOMES and pass deterministically either way.
3. 08 x1 / 0c x1 / 16 x1 (aim-route-3 §4) — untouched, not this task's scope.

## Live checks for the main agent

1. **m8's state-2 branch tick** (resolves note 2): in PCSX2 0700001b, hold ch 193/194 from
   the wave death and watch MissileDeploy[7] (deploy slot 7) + the kind-0x39 object's
   obj+0xf4 for the missile at placement (51.78,−12.84,+11.38) — at which frame does it
   leave state 2, relative to ch 8 (the console channel) and to launch+3600+2399? Expect
   (model): the branch ~8990 abs, ch 8 clear → LAUNCH. If the real game DISARMS with ch 8
   that late, the p0-end/branch timing carries a term the P0_LEN model misses.
2. **The wave's spawn tick**: the astronaut wave's first contact measured ~launch+6650
   (hits from frame 9414 with launch ≈ 2747); the spawn trigger itself is not pinned ([S]
   the m1/m2/m7 p0 sync writes ch 181..188 @t10 [D P0_SYNC] ~launch+6310). A gdbwatch on
   the wave spawner's trigger channel during a held-gate run settles it.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
