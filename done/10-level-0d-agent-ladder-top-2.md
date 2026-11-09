## Your task: `level-0d-agent-ladder-top-2`, aiming freezes the view exactly as the game's does, and 0700000d is walkable on Agent

Branch from origin/s/level-0d-agent-ladder-top, merge current origin/master, and read its docs/reports/level-0d-agent-ladder-top.md plus the Agent ignore message. First its aim-mode audit: read the aim-mode view and cursor update (0x1a4a98 and the pad cursor update) in full with addresses, and settle whether the game freezes the view while the cursor moves in aim mode, and when. If the port differs, fix it in scene.rs's free-aim block in the game's order, pin it through real input on an on-foot level, and re-run every walkreal_*_live one level per tool call, repairing any walk it shifts, gated to its level. Then the walk's levers: the #17 wake lag (delay the second #17 pass 150 to 300 ticks or widen the wake window), then the ladder-top endgame. Helpers stay gated to 0700000d and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run walkreal_0700000d_live, interactables_0700000d_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-agent-ladder-top-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T20:53Z

---
## Report (done, 2026-11-09T23:40Z)


---
## Report from the worker (done, 2026-11-09T23:41Z)

# level-0d-agent-ladder-top-2 — the aim-mode audit settled (the port matches the game; no fix), the #17 wake-lag second pass landed, and the ladder-top levers moved the Agent death from tick 4140 to 8110 (the wire ride); the walk stays red with the ledger and the next levers pushed in its ignore message

Branch `s/level-0d-agent-ladder-top-2`, based on `origin/master` 0a9d39435 (`origin/s/level-0d-agent-ladder-top` is DELETED from the remote — its code changes survive on no ref; this branch was rebuilt from master + its queue report, as the WORKLOG records).

## 1. The aim-mode audit: the game freezes the view exactly while the cursor moves, and the port already does the same — no fix

Read in full, with addresses:

- **[M 0x1a1d28 `Player_Aiming`]** (the pad cursor + view update, 1948 bytes):
  - Gate: pad word `obj+0xdc → +0x60` bit 1 = the aim button; the function runs only in the walk states (returns outside 0x0d..0x0f and on 0x10).
  - **Aim branch, non-scope weapon** (`weapon_data[weapon] +0x68 & 0x40 == 0`): the stick (actions 3/4) **always** integrates the raw cursor `+0x118`/`+0x11c` — `cursor += stick × Plr_AimSpeed_X 0.12808 / Plr_AimSpeed_Y 0.1385 [M ReadTuningVars 0x1d3a80] × FRAME_RATE_MUL × 0.125` — then clamps **±0.296** (0x3e978d50) / **±0.4** (0x3ecccccd). The **view** (`+0x34` yaw, `+0x8a8` pitch) is updated **only from the clamp overflow** (the raw stick × 0.125 when |cursor| sits past the edge), through the accelerated turn speeds `+0x8f4`/`+0x8f8` (ramp toward `Plr_AimTurnSpeed_X 0.05` / `Y 0.06`, ×`Plr_ScopeSpeed_*_Mul 3.0` cap, /`Steps 120` per tick, reset on |stick| ≤ 0.05), divided by the sensitivity `+0x8d0`, yaw ×π/2, and ×0.4/×0.7 underwater. The smoothed turn amounts land at `+0x128`/`+0x12c`.
  - **Scope branch** (0x40 set): the cursor is zeroed and the stick drives the view directly through the scope speeds `+0x8ec`/`+0x8f0`.
- **[M 0x1a7c98 `Player_AutoAim`]**: the smoothed cursor `+0x120`/`+0x124` half-lerps toward the raw cursor while the aim bit is held (`+0x120 += (+0x118 − +0x120) × 0.5`), and decays ×0.5/tick outside aim mode.
- **[M 0x1a4a98 `Player_GetAimingPoint`]**: the aim/bullet ray = head + body matrix × RotX(−pitch(`+0x8a8`)×π/2) × (smoothed X scaled by the viewer, smoothed Y scaled, −1) × 128 — the cursor offsets apply **only when the camera mode `+0x950` == 0** (`+0x950` is the camera mode, [M `Player_SetCamMode` 0x195928]; [M `Player_PositionCamera` 0x18f5d8] switches on it).

**The settled law: the game DOES freeze the view while the cursor moves in aim mode.** The stick moves the cursor inside the box; the view angles move only when the cursor is pinned at the ±0.296/±0.4 edge and the stick keeps pushing; the bullet direction is never frozen — it rides the smoothed cursor through 0x1a4a98.

**The port matches.** `scene.rs`'s free-aim block (the `if self.free_aim` arm) is the same law in the same order: the stick into the cursor (`want[0] = cursor[0] − turn/0.76966`, `want[1] = cursor[1] + look·π/2/0.57724`), the clamp at ±0.296/±0.4, only the overflow into `input.turn`/`input.look` (the view), and the smoothed `cross` half-lerp toward the cursor — the port's mirror of AutoAim's 0.5 ease. The constants are the calibrated cursor↔radian mapping (0.4 × 0.57724 = the measured ±0.2309 rad ray offset at the ±0.4 clamp [probe_0d_bullet]). The signs agree with the game's (cursor right ↔ yaw decreasing in both; cursor up ↔ pitch up in both).

**The prior session's "frozen stick" measurement was the controller's equilibrium, not a freeze.** The trace (`pitch −0.022` frozen, `cross [−0.296, 0.193]` stable) reads exactly as the converged aim: `0.193 × 0.57724 = 0.111` rad of cursor-ray on top of the frozen view pitch −0.022 ≈ the 0.089 target pitch. The act was on target through the cursor while the view stood still — which is precisely the game's law. The walk logs in this session show the same shape on every converged act (`[0d:dbg] ... pitch −0.022 cross [...]` with the ray CLEAR — e.g. agent1.out's bay-duo lines).

**Consequence: no `scene.rs` fix, so the "re-run every walkreal_*_live" step does not trigger.** The walks that exercise the block (every box act in every level) still ran on the final tree where the driver requires them (see §4).

## 2. The #17 wake-lag second pass: landed

The driver's lever, measured into the route (`crates/nf-view/tests/walkreal_0700000d_live.rs`, the d2-gated Agent block only):

- The first #17 pass runs right after the climb; **its own shot is the wake** (d 38.9 from the ladder-top eye (213.1, 18.1, −234.5) to the fire spot (192.3, 19.6, −267.4), inside the P2K hearing radius 40 [L] cited by the prior report).
- The second pass now waits 150 ticks (inside the driver's 150–300 window) and re-picks with the box widened to ±10 (the walk-to-fire-spot approach).
- Measured across every run this session: **#17 landed zero hits** (the t4312 and v3 killing 10.0s are gone) and the pass's box reliably catches the deck sniper #20 instead (down at +209/+303 — the prior branch's known side effect, kept).
- The lag itself is real but LONGER than the window on this RNG: #17 never appeared in the box nor fired in any run — the ledger keeps the pass (it costs 150 ticks when nobody is there) and the next session's ledger watches for him past the bay duo.

## 3. The ladder-top levers (the walk's red blocker)

The rebuilt branch inherited master's OLD order (the prior branch's unmerged route moves are on no ref), and the phase died fast: **death 4140** — the west-roof trio (spawned at the platform-drop wake; the pre-wake census this session reads **no present guard above y 15**, confirming [t3310] "the spawn is the wake — the pre-wake snipe is dead") opened fire at 3292 and Bond died inside the bay-duo pass before the trio legs ever ran.

Levers landed, each measured (all d2/real-damage gated; the plain and Operative twins byte-identical):

1. **The bay-duo-first order** (the run-8110 sequence: #17 pass1 → the bay duo → the wake-lag pass → the trio loop): both bay guards died at +180/+133 **before their engage latency opened** — zero bay drain in that run (the t4312 ledger's 84-damage drain deleted).
2. **The trio loop** (6 passes × cap 450, box ±5, the no-drain act): the prior branch's single 700 cap expired right after the reload's re-converge (+575) — the second clip never fired. The loop's re-picks kill a member whenever a posed-box line clears (+372, +37+133, +37/+133 across runs) and re-pick past a stalled pass instead of burning it.
3. **The second-clip law**: the no-drain act's cap sized so clip 1 + reload + re-converge + the second clip's rounds fit inside ONE pass when the line stays clear.
4. **The wake-lag wait trimmed 240 → 150** — the window's lower bound; the standing ticks feed the trio/bay cadences.

**Outcome: death 4140 → best 8110** — the ladder-top phase now survives (its floor: #33's opening cadence 3292/3532 = 56, before any act can kill, plus #20's 6.0 during the climb) and the death moved to the WIRE RIDE: **#19** (skin 0x5000087, weapon 0x1e, (172.0, 19.6, −292.5)) landed the killing 10.0 at 8110 on the rider at d 49.9 from ~10 remaining health. The worst rolls still die in-phase (4034/4380): the trio act is dice-dependent — it lands +37…+372 when a posed-box line clears and holds fire 450–775 when not — and the RNG re-rolls every run.

**Still red → the ledger and the next levers are pushed in the test's ignore message** (the driver's contract): (a) the wire-ride shooter #19 — a pre-ride box act on (172, −292.5), or accept the ride's one 10.0 by fixing the exit health; (b) the phase floor — a strip-side stand that reaches the west roof pre-climb deletes #33's two unavoidable 28.0s (the strip→ROOF rays are unmeasured); (c) an in-pass retarget (drop a target whose `clear_box_aim` returns None for N ticks, re-pick) converts the 4034-class rolls into 8110-class ones.

## 4. What was run (all on the final tree)

- **`walkreal_0700000d_live`** (the whole binary, `--test-threads=1 --include-ignored`): `1 passed; 2 failed` — the plain master replay **passed** (byte-identical; every change is d2/real-damage gated); the Operative twin failed on the **inherited** slot-hop ledger (`walkreal_0700000d_live.rs:1065`, the jump-drift assert on THIS master — untouched by the d2 gates); the Agent twin failed at its measured ledger death (best 8110 / this roll 4380 — §3).
- **`interactables_0700000d_live`** (`--test-threads=1`): **11 passed, 0 failed**.
- **`nf-game`** (the whole crate, `--test-threads=1`): **1192 passed, 0 failed, 8 ignored**.
- **`nf-formats`**: 109 passed, **3 failed — exactly the 3 `disc_golden` tests** (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`), which the driver allows on the slim ISO; 0 others failed.
- **`nf-audio`**: 33 passed, 0 failed.
- **The full suite via `tools/test/suite.py` did NOT fit this box**: it was started three times (23:00, 23:10, 23:26 UTC); its build phase compiles all four crates' test binaries into a ~7 GB `target/release` on a **9.9 GB overlay rootfs** that shares the disk with the ISO (1.0 GB), the repo and the toolchain — it died with `No space left on device` (os error 28) and `ld terminated with signal 7` mid-link. Space triage (cache deletions, `work/decomp`, the already-run nf-view test executables) freed enough to run the crates above, but not to relink the whole nf-view test set again. Per the driver's precedent ("The full suite did not fit" is fine): the diff touches **only `crates/nf-view/tests/walkreal_0700000d_live.rs`** — no `src/`, nothing under `tests/common/` — so no other binary's behaviour can change; both of that file's binaries and three whole crates ran green (modulo the documented ledgers).

## Live checks for the main agent

1. **The aim-mode law, one probe** (settles the audit on the real game): in PCSX2 on 0700000d, aim the P2K and HOLD the stick right WITHOUT crossing the ±0.296 cursor edge, then cross it: gdbwatch BL+0x118/+0x11C (the raw cursor) and BL+0x34/+0x8a8 (the view yaw/pitch) — expected: the cursor moves and the view angles stay frozen INSIDE the box; once |cursor| ≥ the edge, the view angles start moving. Then BL+0x120/+0x124 (the smoothed copy) should follow the raw at ~0.5/tick.
2. **The wire-ride shooter**: at the ride, gdbwatch drone #19's state and Bond's health — expected: the game's own rider eats the same single-shot cadence from (172.0, 19.6, −292.5) (the spot is a placement read [L census], the fire law is the game's own [M NDrone2_DSTATE_SniperFire 0x15fd88]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for this session.
