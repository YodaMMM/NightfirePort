## Your task: `level-0d-agent-ladder-top-4`, Countdown can be walked past the ladder top on Agent with real health

Branch from origin/s/level-0d-agent-ladder-top-3, merge current origin/master, and read docs/reports/level-0d-agent-ladder-top-3.md and the Agent walk's ignore message in walkreal_0700000d_live. The death moved from tick 8110 to 13291: the ride, deck and descent pass. The settled law: the west-roof pair g32/g33 cannot be killed from the ladder top (torso lines blocked, head boxes graze at d 63). Take the report's next levers in order: avoid the pair's lines instead of trading with them, or close to d ~31 where the head line lands. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000d and the difficulty. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000d_live and interactables_0700000d_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0d-agent-ladder-top-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T09:12Z

---
## Report (done, 2026-11-10T10:07Z)


# level-0d-agent-ladder-top-4 — the death HOLDS AT THE ANNEX DROP (13700): the ride, the deck, the E, the walk-off, the catwalk, the descent, the roof lane and the annex landing all pass on Agent real health; the DECK #21 KILL landed (the decisive drain's shooter dies at +47 down his own proven line); the pool arrives at 10.0 and the pinned annex fall kills on the landing tick

Branch `s/level-0d-agent-ladder-top-4`, cut from origin/master 5ae2a5de8 (the top-3
branch is deleted from the remote — its changes were merged to master as 5ae2a5de8, and
its report was read from the queue `done/10-level-0d-agent-ladder-top-3.md`).

## 1. What the baseline audit settled (the ledger's lever (i), answered with evidence)

The baseline run (this tree, the top-3 state) reproduced the 13291 death exactly. The
band sweep's CHEST act audit settled WHY #21 was never landed from the ladder top: the
act **converged** (cross stable at [−0.296, −0.237], pitch −0.110) and then **held fire
3 × 600** — the chest line to the settled band spot (163.3, 1.3, −214.0) from the
ladder-top eye (213.1, 18.1, −234.5) dies at **0.982** (full / static / bullet_scoped
all 0.982) on the ~1.7 jamb. The top-3 report's "audit the sweep's convergence" question
is answered: the convergence was never the failure; the line is geometrically dead.

## 2. What was measured dead (runs 2 and 4)

1. **The ladder-top keyhole (the box-act variant, run 2, MEASURED DEAD).** The only clear
   line from the top threads UNDER the ~1.5 wall top into the y-0.3 fire-pose box —
   `clear_box_aim`'s part-5 head box drops to y 0.3 in fire anims (the deck #16 pose law
   [M NDrone2_Collision 0x140e90:91-121]) and its centre line passes under the wall
   (the audit: full/static/bullet_scoped all CLEAR at the y-0.3 aim point). The margin is
   centimetres: the act's tap gate (vert 0.25 u → the 0.006 rad clamp floor = 0.32 u at
   d 54) lets the ray pass 0.32 u off the centre — over the box and into the wall top
   3 u short. **6 clear-line scoped rounds ate the clip for ZERO.** The chest act stays
   (its blocked line holds fire; the cap is the only cost).
2. **The roof-lane return leg (run 4, MEASURED DEAD).** The lane end stand
   (196.3, 20.3, −278.9): the return ray to the band dies at **0.014** on the shed's east
   wall (normal (1,0,0), 0.9 u out). Removed; a comment marks the measured state.
3. **The crouch-dodge shield is LINE-SPECIFIC (runs 3-7).** It held the catwalk line —
   run 3's killing −10.0 at 11122 (191.50, 13.22, −265.71, mid-transit) never returned
   once the transit ran crouched — and did **NOT** hold the deck line: the 0.45 target
   drop [M Player_GetHeadPos 0x195aa0:39-51, the BL+0x90C drop; the visibility 0.75 ×
   factor under the 0.7 LOS gate [M Drone_ProcessOpponents 0x1390c0:40-42 + M
   0x176a28]] is 0.024 rad of cone at the rim's d 19 but **0.007 at d 67** — inside the
   cone either way. The crouch windows stay (deck statics + catwalk legs) with fresh-stand
   presses bracketing them (the walk-off needs the run's carry; the descent grab runs the
   baseline standing shape). All `difficulty >= 2 && real_damage` gated.

## 3. What LANDED: the deck #21 kill (runs 5-7)

The ground-band sniper #21 (skin 0x5000087, weapon 0x1e) lands the decisive 28.0 INTO the
deck phase every roll — his shot INTO the deck (frame 9622 in runs 4-7; the baseline's
11621 into the roof lane) is the **reverse-ray proof** that the deck→band line lives.
From the deck arrive spot (199.25, 14.16, −260.16) the north-west return line crosses
UNDER the mid-band obstruction (its ~10 y at z −250 vs the structures' 18-20) and never
touches the shed (z −269.4 is south of the segment). The **FAST act** (the 160-tick
aim-off drain skipped [M Player_SetWeaponAnimObj 0x1a2e38 case 0xf — the same race law
the run-4380 ladder-top variant was built for]) killed him at **+47** (the drained act
took +207 and lost the parting round by ~10 ticks). The leg runs FIRST at the deck,
before the #20 fight, and holds fire unless `clear_box_aim` returns a real box centre.

**The parting round:** #21's 28.0 still landed at 9622 in EVERY variant (the drained
+207 kill, the reordered leg, the fast +47 kill) — the round was already in flight when
he died; the damage record's weapon field reads 0x0 (the corpse's reset field) at
impact. The kill is real and proven; the burst is ~200 ticks faster than the earliest
possible deck kill.

## 4. The pre-wake census, wide (the ledger's lever (iii), ANSWERED)

The new log-only census (present + alive, y > 11, z < −200, full coords/skins/gates,
printed pre-wake) reads **exactly three** drones: g11 (0x5000093, (214.21, 12.09,
−289.44), enabled FALSE — a bay placement), g20 (0x5000087, (196.90, 14.04, −264.10) —
the deck guard), g22 (0x5000090, (204.84, 12.09, −290.05), enabled FALSE — a bay
placement). **No west-roof placement exists pre-wake** — the trio spawns at the
platform-drop wake [t3310 confirmed on this tree]. The act-proof pair (g32/g33) cannot
be deleted pre-wake from any stand the route reaches; their 56 opening cadence (3292/
3532) is structural under this route shape.

## 5. The pool ledger at the death

100 − 56.0 (g33's opening cadence, frames 3292/3532) − 6.0 (#20's climb graze, 3164) −
28.0 (#21's parting round, 9622) = **10.0** → the pinned annex fall (landing timer 76 →
raw 16 → ×0.7; the assert pins `annex_fall == 76` [M 0x1949a8:117-126 + 0x1902c8:47-60])
− 10.0 → 0 at tick 13700, (197.95, 10.56, −278.88).

**Next levers, in order (also in the ignore message):** (i) kill #21 PRE-RIDE — the only
window his death beats the parting round; the pre-ride stands are the keyhole (dead,
§2.1), so the shape is a tight-tolerance ladder-top act (a ~0.15 u tap box instead of
the 0.006 rad clamp floor's 0.32 u) fired during his fire anims while the part-5 box
sits in the under-wall window; (ii) shave ~200 pre-deck ticks so the +47 kill lands a
full burst earlier (the settle poll 30 + the assert are already minimal; the ride's
~1370 and the pre-ride legs are the budget); (iii) the Operative twin's own 13800
slot-hop ledger stays the level-0d-operative-walk-2 task's.

## 6. What was run (all on the final tree)

- **`walkreal_0700000d_live` whole binary, `--test-threads=1`, `--include-ignored`:**
  3 tests ran — the **plain master replay PASSED** (byte-identical, 20.84 s), the
  **ledger twin failed only its own inherited 13800 slot-hop assert** (end health 46.4,
  its documented state, untouched), the **Agent twin failed at its measured ledger
  death** (13700, §5).
- **`interactables_0700000d_live`** (`--test-threads=1`): **11 passed, 0 failed** (28.84 s).
- **The full four-crate suite via `tools/test/suite.py` did NOT run in this box**, on two
  independent grounds: (a) the sandboxed background runner dies between tool calls here —
  two starts (`nohup`, then `setsid nohup … & disown`) both left a 0-byte `suite.out`
  and no process, the same documented death the live-backlog session recorded; (b) the
  disk math the top-3 session documented still holds — 5.4 GB free at build time vs the
  ~7 GB `target/release` the four-crate build needs. Per the driver's precedent ("The
  full suite did not fit" is fine): the diff touches **only
  `crates/nf-view/tests/walkreal_0700000d_live.rs`** (+ docs/WORKLOG.md and this
  report) — no `src/`, nothing under `tests/common/` — so no other binary's behaviour
  can change; the binary that owns the changed file ran whole (3/3 tests), and
  `nf-view`'s other 0d suite (interactables) ran whole too.

## Live checks for the main agent

1. **The deck→band return line** (settles the kill line's geometry on the real game): on
   0700000d at Agent, from the rotate crane's deck arrive spot (199.25, 14.16, −260.16)
   aim the P2K at the ground-band sniper in the band around (158, 1.0, −225.5) — the
   port's scoped pick reads the line CLEAR and the kill landed at +47 (runs 5-7);
   gdbwatch the bullet spawn and the impact — expected: the game's own round lands the
   same way (if the game's round dies on the mid-band obstruction, the port's scoped
   pick deviates on shallow down-lines and `probe_0d_bullet` needs a re-look).
2. **#21's parting round** (settles the in-flight law): gdbwatch the band sniper's fire
   script during the deck phase — expected: the round spawns ~200 ticks before the deck
   kill's impact and is already airborne when he dies (the port's bullets outlive their
   shooter, `Bullet_update` 0x12e7b0 has no owner-liveness gate [M]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures
for this session.
