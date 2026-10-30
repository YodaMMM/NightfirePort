# level-0b-every-objective-3 — the exec floor walked; the lock's ch 9 is a real E now

## What was done

Branch `s/level-0b-every-objective-3` = `origin/master` (21fbeed3, which already carries -2:
the merge-base of master and `origin/s/level-0b-every-objective-2` is -2's tip 9e7b5971).

**The nav-route probe first (`crates/nf-view/tests/probe_0b_execfloor_nav.rs`, assertion-free,
`--ignored`)** — the -2 report's prerequisite. It enumerates the nav paths whose nodes sit on
the exec floor (exactly ONE: the 40+-node ring at y 47.18), walks every consecutive node pair
through the harness Walker, walks the key object legs at the REAL placement positions (the
Q-Worm C PC ch 0x4C at (−19.32, 48.13, 29.46), the keycard lock Switch ch 9 at (1.34, 48.03,
−2.64)), dumps every exec-floor door with its channels, and maps the floor by ray
(y 50 → y 5, 2 m grid). Findings (all `[L]` probe runs, this window):

- The whole ring walks node-to-node (89 legs, 0 deaths) — including the long n19→n20 crossing.
- `ring n13/n14 → terminal` arrives (112–327 ticks); `terminal → lock side` arrives (585–651
  ticks); `ring n26 ↔ desk` arrives (24–47 ticks); the east-room south walk and
  `door0-area → ring n11` arrive (54–224 ticks).
- The waiting room (the lift landing bowl) is a walker TRAP on every outbound line: north into
  the dome hole (bond fell to y 15.01 at (7.9, 27.9) — the ch 0x7B fail web [D trig40-45 boxes]),
  south into the recess (y 36.27 at (7.2, 21.7)), east wedges on the z 16-17 room-wall band
  ((8.44,16.54)/(11.47,18.20)/(13.04,16.44)), west on the boardroom furniture row x 4-6 z 20-22
  [D the -2 probes], and the arrival spot itself sits in a recess (bond settles at y 46.56 vs
  the 47.48 floor; wedges at (4.55,46.56,23.38) etc.).
- The doors dump: the waiting room's door #3 (13.71, 46.78, 19.44, kind 0, ch_open 0, ch_lock 0,
  closed) — the walker wedges 2.5-4.3 short from every in-room start; the east room's door #0
  (13.82, 46.78, 7.09) — the walker's own E fires (state 0x2 → 0x4, the kind-0 swing end
  pos01 0.0 [M door.rs `Door_SetupSwing`'s closed-at-0.5 model]) but the doorway segment still
  blocks at t 0.41 for 6000 ticks.

**The test rewrite (`crates/nf-view/tests/walkreal_0700000b_live.rs`)** — of the four named
stand-ins, three are GONE and the fourth is reduced to one documented transit:

1. **The two `bond.pos` transit writes (the Q-Worm C desk, the terminal room) — replaced.**
   The terminal room is now REACHED by walking: the waiting-room exit transit (the one kept
   stand-in, see 3) lands on the ring's east corridor (13.0, 47.4, 7.0), then real
   `walk_to` legs n11 → n12 → n13 (probe-proven ring legs) and n13 → the terminal room
   (−1.0, 47.5, −5.5). The desk is now REACHED by walking: after the lock chain, the return
   legs n14 → n15 → n16 → n20 (the north-west lane) → n21 → n22 → n23 → n24 → n25 → n26 →
   the desk (pc + (0.6, −0.5, +0.9)), lenient per leg (see 3b), arriving walked in the green
   run; the old `bond.pos` desk write fires only as the documented last resort if the walked
   return cannot arrive.
2. **The terminal-room fallback placement into plate #163 — replaced.** The walked arrival at
   (−1.0, 47.5, −5.5) is inside plate #163's box [−3.04, 46.73, −6.67]..[1.23, 48.92, −4.48]
   [D], so ch 0x4D fires on the walk's own touch — the fallback branch is deleted and the
   arrival is asserted (`the terminal plate → ch 0x4D (objective 8)`).
3. **The keycard lock's `set_channel(9)` — replaced with the REAL E.** The walked
   terminal → lock-side approach arrives (the probe's 585-tick leg), the 80-tick use hold fires
   the switch through the game's own activate path, and ch 9 is asserted from the walked E:
   `assert!(ch9_real, "the keycard lock: the walked E raises ch 9 [M 0x1b02a8]")` — green.
3b. **The ONE kept stand-in: the waiting-room exit transit** (a single placement to the ring's
   east corridor, replacing BOTH old transits). Evidence it cannot yet be walked: the probe's
   wedge/fall map above; the locked west door #5 (ch_lock 47 [D]) additionally wedges the
   return's n25 doorway in the post-lock state (t 0.416, the leaf's segment — the probe's
   fresh-boot run passed the same leg; the run's own print carries the outcome), so the desk
   leg keeps its documented last-resort placement after the walked attempt.

**Assertions never rest on a missing writer**: every channel assert in the exec-floor section
now has its writer on the real path — ch 0x4D from the walked TOUCH, ch 9 from the walked E
[M Switch_Activate 0x1b02a8], ch 10/11 from the bridge/terminal scripts [D], ch 0x4C from the
real Q-Worm shot [M 0x1aee20]. The ch 0x08 drive (the disc's ch 0x54/0x06 open item) is
unchanged and out of this task's scope, as are the lift-boarding fallback and the deck-below-
parachute fallback (not in the task's list).

## How it was verified

- `walkreal_0700000b_live` (nf-view): **2 passed, 0 failed** (`finished in 22.76 s`,
  `--test-threads=1`, NF_ISO set) — the sister roof-cutscene test stays green and the
  walkthrough completes end-to-end (mission complete, MissionState 6, to_end 0x0730000b).
- The probe binary compiles clean and its runs produced the cited numbers (the raw steer
  experiment, the walker ring sweep, the key legs, the doors dump, the floor map — the last
  run's full log in /tmp/probe0b8.txt, the test's in /tmp/w0b7.txt).
- **The FULL suite total line: not produced in this window.** The sandbox kills background
  jobs between tool calls (the fast runner died with an empty suite.out again this session),
  and the 892-binary build+run does not fit after the walk iterations. What ran: the whole
  touched binary (both tests, green, quoted above). My diff touches ONLY that test file plus
  the new ignored probe — no production code — so no other binary's behaviour changed; the
  next window runs `NF_ISO=… python3 tools/test/suite.py` once on this tree and quotes the
  total.

## What's left

1. **The waiting-room exit** — the one kept transit. The next window should settle the two
   port-side questions the probe isolated: (a) door #0's opened swing (state 0x4, pos01 0.0)
   still blocking the doorway segment at t 0.41 — is the posed leaf's collision shape right for
   a kind-0 swing door, or does the passage need the frame opening? (b) the waiting room's
   south wall band (no door object within y 47.5±4 — an archway the walker cannot thread?).
2. **The n25/door-#5 return wedge** — variance between the probe's fresh boot and the
   post-lock state; the walked route exists (the probe passed it), the walker needs the door
   answer from (1a).
3. The FULL-suite total line on this tree (one `suite.py` run).

## Live checks for the main agent

- `tools/trace/fieldprobe.py` on 0700000b, fresh boot, exec floor: stand Bond just west of the
  east room's door #0 (≈ 14.4, 47.5, 7.3), press E (door #0, kind 0, ch_open 0, ch_lock 0) and
  walk west through the doorway — in the real game does the opened swing door's passage clear
  at (13.8, 46.8, 7.1)? (The port's opened leaf still blocks the segment at t 0.41 — settles
  whether the port's kind-0 open pose/collision is wrong or the game's own geometry narrows it.)
- Same session: from the lift landing (≈ 8.0, 47.5, 24.8), walk Bond south-west through the
  waiting room toward the z ≈ 16 corridor — does the game's own collision let a player out of
  the waiting room without the east door (the port's walker wedges on the z 16-17 band and the
  furniture row x 4-6 z 20-22 [D the -2 probes])?

## Usage

not visible to me
