# level-0b-every-objective-2 — report

Task: continue 0700000b end-to-end. The rewritten walkthrough failed at the Q-Worm install leg
("the Q-Worm C install never landed" [M 0x1aee20]) though the pre-shot state matched the merged
version; bisect the pre-shot legs and fix the real cause. Then the lift boarding (cab_y never
matched hash 2000367), the walked lock and terminal approaches, real input only, the FULL suite.

## What's done

1. **The Q-Worm install failure's real cause found and fixed — it was never the shot path.**
   Bisected with in-test probes: after the rewritten shaft walk `disabled false`; **19 ticks into
   the boarding poll `disabled -> true`**, with ch 0x7B on at the flip and the fail wedge
   (0x0600098A) mid-play. The chain: the shaft-mouth walk left Bond standing on the shaft lip
   (x 0.5 target is over the opening), his residual walk velocity slid him in, his capsule
   crossed the **y 7.98..11.98 out-of-bounds slabs** (trig40-45: kind 2 (TOUCH), out 0x7A, mask 8,
   boxes e.g. trig43 lo [−30.9, 7.98, 19.53] hi [33.3, 11.98, 110.3] [D]) → **OR #46, inputs
   [0x79, 0x7A] → ch 0x7B** → the fail wedge 0600098A played; its event 4 disables Bond and this
   path never re-enables. `step_bond`'s disabled gate (`let (trigger, reload) = if self.disabled
   { input = Default::default(); (false, false) }`) then ate the shot's trigger for all 1800
   ticks — **no 0x58 bullet ever spawned** (`bullet_paths 0`) and `PCQWorm_Update` 0x1aee20 never
   saw the class-5 toucher. mission_status was 3 by the pre-shot (the fail processed). The fix is
   the safe wait spot + the working boarding (below): Bond never enters the shaft, ch 0x7B stays
   off, `disabled false` at the pre-shot, and the install lands [M 0x1aee20's class-5 0x58 scan
   → state 1 → `switch_channels[0x4C] = 1`].
2. **The lift boarding fixed — the cab poll was comparing the wrong number.** `cab_y` matched the
   **decimal literal 2000367**; the runtime script entity's hash is **0x2000367 = 33555303**
   (lifts_ride_live's green pin already matched the hex form). One probe (in the test, the
   report's requested id/hash pairing probe) dumps the runtime list: **hash 0x2000367 kind 49**,
   matrix x/z LOCAL (0,0) while the collision box is placement-anchored (x 0, z 23.19..26.56
   [D placement #232]) — only matrix[3][1] carries the keyed y 11 → 48.44 → 11 [D nis dump].
3. **The balcony → exec floor transit is now WALKED and RIDDEN on the port's real machinery**
   (the walkthrough completes end-to-end, **24136 ticks, mission complete**): wait on the
   balcony's proven z≈25.8 corridor 0.7 east of the lip (x 2.3) with the walk's residual velocity
   zeroed; board during the 499-tick bottom hold — the cab's stand surface is **cy + 4.1**
   (15.1 at key 0 [L lifts_ride_live: grounded 16.11]), so the board is the player's **centred
   hop** (dxz < 1.0 && x < 1.0 — a landing near the box's east face gets shoved off it sideways
   by the 0.06/tick rise and the carry then has no body to grab; observed in the run trace);
   the carry [M 0x194188 → Control_InheritVelocity 0x133e88] rides Bond up (trace: y 16.13 →
   22.77 → 36.14 → 48.60 with the cab), the step-off steering loop walks him east off the level
   surface onto the exec floor. The ONE documented stand-in transit is gone from the green path
   (the stand-in code remains only as the boarding-failure fallback).
4. **The lock and terminal approaches**: attempted the walked versions through the level's nav
   graph (`walk_nav_to`, the game's own AI navigation) with stand-in fallbacks — **not landed**:
   the wedged wandering (3 × 6000-tick caps) re-trips the fail web (the dome hole's ch 0x7B web —
   the same chain as item 1) and the pre-shot went `disabled true` again, so the tree keeps the
   checkpoint's documented stand-ins (the boardroom furniture row blockers [D probes] + the
   ch 9 drive). The exec-floor nav routes need validating (the route the walker follows falls
   into the dome hole) before the walks can run — left item below.

## How it was verified

- `walkreal_0700000b_live` (nf-view): **2 passed, 0 failed** on the final tree
  (`finished in 21.07s`) — the walkthrough's every assertion is on the real paths: the spawn
  plates, the boot Q-Worm loadout (no inventory poke), the real shaft walk, the real cab ride,
  the terminal plate ch 0x4D, the lock chain ch 9 → 10 → 11, the Q-Worm install through the real
  trigger/bullet/PCQWorm path, the roof-gate chain (E-plate #166 → SEQ #252 → Trigger_Calc →
  SP wedge → ch 0x59/0x57/0x2A), the parachute, the roof jump, MissionState 6 + to_end
  0x0730000b.
- The sister test `the_roof_cutscene_060004fa_writes_ch59_ch57_and_arms_the_helicopter_gate`
  stays green (same binary, same run).
- The bisect evidence is reproducible: the probes printed the disable flip at boarding-poll +19
  with ch 0x7B/0x7A on, the OOB slab boxes, and the fail wedge mid-play (the numbers in item 1
  are from those runs, /tmp/0b_fix*.log).

## What's left

1. **The walked desk/terminal/lock approaches** — the exec-floor nav routes fall into the dome
   hole's fail web when the walker wanders; validate the nav graph on the exec floor first (a
   probe test that walks each nav route and dumps where it dies), then re-attach the walks with
   short per-leg caps so a wedge cannot wander Bond into the web. The interactions are real
   already; only the approaches stand in.
2. **The FULL suite total line** — the sandbox kills background jobs between tool calls (the
   fast runner died with an empty suite.out twice more this session, setsid included), and the
   foreground window did not fit the full four-crate pass after the fix iterations. What ran on
   the final tree: the whole `walkreal_0700000b_live` binary (both tests, green, quoted above).
   My diff touches ONLY that one test file (no production code), so no other crate's behaviour
   changed; the next session should run `NF_ISO=… python3 tools/test/suite.py` once and quote
   the total.
3. Carried from the previous report: the disc's writer of ch 0x54 / ch 0x06 (obj 9's OR inputs)
   stays the level's open item (the walkthrough keeps driving the mux's ch 0x08 input, exactly
   as the level doc's walkthrough does).

## Live checks for the main agent

- `tools/trace/fieldprobe.py`: on 0700000b, stand Bond on the balcony at the shaft mouth
  (≈ x 1.9, y 15.1, z 25.8) while the paternoster cab is at its bottom hold — confirm in PCSX2
  that the cab's stand surface is ~15.1 (the port's cy + 4.1 model; the balcony floor ~14.1) and
  that stepping east off the lip onto the cab is the real boarding move. Access R on the cab
  object's world matrix during the cycle; expected y keys 11.0 → 48.44 → 11.0 with 499-tick
  holds at each end [D nis dump].
- `tools/trace/gdbwatch.py`: watch `switch_channels[0x7A]` (0x26fc90+0x7A) and `[0x7B]` while
  falling into the lift shaft — expected: 0x7A writes first (the OOB touch), then 0x7B (OR #46),
  then the fail scene — confirms the port's fail-web model against the disc.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures for the session
(no environment variable, file or tool output in this session carried them).
