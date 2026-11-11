# real-health-14-00agent-9 — the 07000014 00 Agent walk: the head law, the strict stands and the restock landed; the guard-sight drop pocket re-rolls with the clock and the ledger + the ordered levers are pinned

Branch `s/real-health-14-00agent-9` = origin/master 784ba2d99 + the levers in
`crates/nf-view/tests/walkreal_07000014_live.rs` (all file-local, all `!tops_up()`-gated — the
difficulty-1 walks are byte-identical to master). Status: **the named base
`origin/s/real-health-14-00agent-8` is GONE from the remote** (only `-4` and `-6` survive;
GitHub events/PRs hold nothing of -7/-8), so the window continued from master's own state,
where the d3 walk carried the 1.72-landing wedge ignore. On this tree the wedge is GONE — the
descent lands on real input and the fights answer — but the guard-sight tree opened a NEW
pocket at the same slab (the melee closer below the lip), and the walk is still red with the
fresh ledger and the ordered levers pinned in the ignore message.

## What's done

1. **The trench head round (the margin lever, the -6 chain's law carried).** `shoot_head_loose`
   (file-local, from `origin/s/real-health-14-00agent-6`, the shared helpers untouched) rides
   the part-5 box centre with the game's own `t_box < world_t` entry comparison
   [M 0x1ea398:216-229; walker.rs `exposed_aim`'s head law], torso +0.3 fallback, the caller's
   `shoot_loose` as the backstop. One P2K head round is 35
   [M NDrone2_HitDamage 0x145558 × DroneDamage_Head; the P2K's 3.5 → 35]. The trench guard's
   kill went from **12 rounds (clip 3 spare 9 → 0/0) to 1 round, killed at +20 ticks**
   [L run3/run7 ledgers] — the clip survives for the descent's fights.
2. **The strict stands.** `agent_clear_blockers`' stands fire through the strict head gate on
   the real-health runs: a corner-blocked guard costs ZERO rounds (this window's measured
   waste: the shared stand burned the whole conserved clip — 11 rounds — on the stairwell #1
   "killed false" through the corridor corners [L run2]). Cap 150 ticks, not 700: the head
   window either opens at once or the corner blocks it for good, and the 700-tick aim phase
   re-rolled the descent's closer into the drop path [L run8/9 vs run3].
3. **The sisters' clears fire head-first and loot-less.** `clear_hostiles` keeps master's
   loot-pass + torso-gate order for the tops_up walks; on `!tops_up()` it fires the head round
   first (one round a 12-hp guard, one round the 28-hp closer) and does NOT run the loot pass —
   the measured death: the dry-gun clear's loot pass walked Bond off the landing into the
   closer's melee pocket, 70.0 for #6, death at tick 4960 [L run2].
4. **The restock lever (the task's named lever), in place.** After the drop lands on the low
   floor the walk calls the walker's own `resupply` BEFORE the machine-room clear engages —
   rounds before the fights, not a fetch inside them (the measured death fetched mid-fight at
   d 1.7 under the closer's bursts [L run2]). Plus the pre-drop clear is ammo-gated (skipped
   whole when the in-hand gun is dry).
5. **The fights' head-hunting works when the fights engage.** The south-row #56 fight answered
   and killed at ZERO cost (health 50.0 → 50.0) [L run3] — the return-fire machinery with the
   head-hunting pick is the right answer once the shooter is pickable.
6. **Measured negatives, reverted, and recorded in the ignore note:**
   - the fights' unseen-cover opt-in (`with_real_damage_unseen_fights`) answers the
     under-the-lip shooter during LEGS but its creep walked Bond BACK INTO the burst band
     mid-retreat (the tick-3825 death at (31.72, 1.76, 35.01), d 0.9) [L run7] — reverted;
   - a nav leg back to the pad west dies the same way — the fights live in the legs;
   - the pad standoff dance on bare input (backpedal out of the under-3.0 band
     [M DroneWeap_DoBulletAccuracy 0x179740:113-115]) — same 3825 death: the walker's tick
     machinery moves Bond during bare ticks too, the dance never converged.
7. **The in-hand ammo law (the key discovery for the next window).** At the landing
   `sim.gun` (the IN-HAND gun) reads clip 0 spare 0 while the P2K's ~11 rounds sit in the
   INVENTORY [L run9's pre-drop dump: `armed false ... clip 0 spare 0` right after the trench
   head kill and the zero-round strict stand]. The armed gate must arm through the real switch
   (`switch_to_armed`'s cycle) before any clear — the current in-hand check skipped the clear
   and left the head round unfired.

## How it was verified

Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), `git status --short` checked
before the commit, only the task's files staged:

- `walkreal_07000014_live` whole: **7 passed / 0 failed / 3 ignored** (95.36 s) — the d1
  plain walk, the vent drop, the tripbomb route, the cascade and the rest green; the 3 ignored
  are the d2 real-damage Agent sister, the d3 00 Agent walk (this task's ledger) and the
  real-damage sister — all byte-identical gates to master for the tops_up paths.
- `interactables_07000014_live`: **10 passed / 0 failed** (15.14 s).
- The d3 walk's measurement runs: 10 whole runs this window (run1-run10), each quoted above
  with its ledger line.
- **The fast suite (`tools/test/suite.py`) was started in the background at 03:31 but died
  silently mid-window** (the process vanished, `suite.out` empty) — the background-job kill
  between tool calls is the likely cause. The full suite did not run this window; the two
  014 binaries above are the direct coverage, and every changed gate compiles to master's
  identical expression when `tops_up()` is true.

## What's left

The ignore message on `walk_the_infiltration_to_the_delta_exit_real_damage_00agent` carries
the full ledger and the ordered next levers:
1. arm through `switch_to_armed` at the landing (the in-hand ammo law), then the head round at
   the closer from the pad west (the pad's eye ~4.3 passes over the slab top 2.4 — the sight
   line the lip blocks from the landing);
2. the pad standoff dance — but on the walker's own leg machinery, not bare ticks;
3. the unseen-cover opt-in scoped to the descent's legs only (it answers the under-the-lip
   shooter where the plain pick has no candidate);
4. the clock-independence question: the pocket re-rolls #6's charge state with every upstream
   timing change (the stand's 150 vs 700 ticks moved the death from tick 4624-shape to
   3673/3825-shape) — pin the descent's tick count first, then tune against ONE state.
Then the d2 wrap (`walk_the_infiltration_to_the_delta_exit_real_damage_agent` + the
`with_real_damage` wrap) is the one-liner it has been since master — parked on the same
drop-west pocket crossing.

## Live checks for the main agent

1. **The under-slab lip sight line (07000014, the 1.72 landing's west edge)** — carried from
   the -6 report, now sharpened: from Bond's stance on the landing (33.87, 2.28, 35.36), does
   the REAL game's Bond have a firing answer to the guard standing below the lip at
   (32.4, 1.8, 34.4)? A `fieldprobe.py` read of the guard's health while the player fires from
   the landing edge (10 s) plus a screenshot would settle whether the port's lip geometry
   (every down-shot dies on the slab edge [L run4's ledger: 30 hits taken, hp 28 untouched])
   matches the game's.
2. **The in-hand vs inventory ammo display** — with the P2K owned and ~11 rounds carried, what
   does the REAL game's HUD read after the claw is drawn (the port's `sim.gun` reads the
   in-hand gun only [L run9])? Decides whether the walk's armed gates should read the
   inventory (the port's `Inventory::clip_slot` law) everywhere.

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in
this environment's interfaces for this session.

## ntfy failed

All three fallbacks failed (git push to github.com works from this sandbox; ntfy.sh does not):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within 30 s, killed by the tool timeout (no response from ntfy.sh).
