# level-gates-player — the per-level gate census: 11 of 14 sites PORTED, 2 fixed this window (the party slow-walk ×0.9, the 07000012 void-fall 0x7C), 1 has nothing to gate

Branch `s/level-gates-player` (from `origin/master` 38fd3151e). Two `crates/*/src/` files
changed (`crates/nf-game/src/player.rs`, `crates/nf-view/src/scene.rs`), each fix in its own
commit, plus two tests.

## 1. The census — every `GameState._12_4_` level-code branch in the set

The set (every `Player_*`, `Bullet_*`, `Gun*`, `Weapon*`, menu and HUD function in
`work/decomp/ACTION.ELF/`) holds **14 comparison sites in 10 functions** (the sweep also
checked switch-statements, reversed operand order and cached-alias reads — none exist in these
files; two grep hits are 128-bit vector spill fields, listed at the end). One line each:

- `Drone_ModBulletDamage` 0x139f50:30 — level < 0x700000c (folded with the anim-set ≠ 0x12
  test) — **PORTED** (crates/nf-game/src/drone.rs:12175).
- `Drone_ModBulletDamage` 0x139f50:32 — level < 0x700000e — **PORTED** (same gate).
- `Drone_ModBulletDamage` 0x139f50:37 — level != 0x700001b — net C semantics "anim set 0x12 AND
  level != 0x700001b" — **PORTED** (drone.rs:12175; the C's three-site fold is equivalent).
- `HUD_UpdateSpacePane` 0x19d3b8:13 — level != 0x700001b → the pane never arms — **PORTED**
  (crates/nf-view/src/hud.rs:72, wired live at hud.rs:1316).
- `HUD_UpdateSpacePane` 0x19d3b8:18 — the leave-level clear — **PORTED** (the same single gate).
- `DroneWeap_DoBulletAccuracy` 0x179740:63 — level < 0x700000e — **PORTED** (drone.rs:12151).
- `DroneWeap_DoBulletAccuracy` 0x179740:64 — 0x700000c/0x700000d + anim set 0x13 → the accuracy
  bonus — **PORTED** (drone.rs:12151-12154; the C file is a partial decompile, the gate reads
  clean).
- `MenuManager_Update` 0x21c3d8:45 — level 0x7000048 → the memory-card pump — **PORTED
  structurally** (crates/nf-view/src/menu.rs:4399-4405 — the port's menu machine only runs as
  the front end, so the level compare is architectural; ParisCard/PS2_DoCardFlow implemented).
- `Player_Weapon` 0x1a24c8:243-251 — 07000007 + ch 0x48: the weapon-cycle degrade and the
  0x48-rising-edge auto-draw — **PORTED** (player.rs:2634-2653, wired at
  crates/nf-view/src/lib.rs:5483/5687).
- `Player_HandleJump` 0x1916b0:106 — 07000007 + ch 0x48 clear: the jump refusal + sound 0x1db —
  **PORTED** (scene.rs:3456 `jump_locked`, player.rs:1679-1680, commit 8e72343f8).
- `Copter_FireGun` 0x17d828:87 — level 0x7000008 → the copter bullet weapon index 0x61 vs 0x71 —
  **PORTED** (crates/nf-game/src/copter.rs:757, the level flag threaded at :597, pinned by
  copter.rs:1291).
- `Player_Move` 0x191dd0:139 — **MISSING → FIXED this window** (§2).
- `Player_CollisionHandler` 0x1949a8:104 — **MISSING → FIXED this window** (§3).
- `MenuManager_Monitor` 0x21d508:47 — **MISSING, nothing to gate**: on 0x7000048 the branch
  skips the psiMovieFinished + counter_152 controller-hotplug/attract block — and that guarded
  block is itself unported (the manager movie counters, the multitap scan and the
  0x4000002f controller-disconnect manager have no port counterpart; gamepad.rs:33 notes the
  multitap bring-up explicitly). Porting the gate alone would gate nothing; the fix is the
  block's own port task. Left, with this scope note.

Out-of-scope grep hits (not GameState level reads): `001916b0:173` and `0017d828:69`
`auVarN._12_4_ = uStack…` are vector spills; the other `_NN_4_` GameState fields in these files
(frame counter, difficulty) are different gates.

## 2. `Player_Move` 0x191dd0:139-141 — the party slow-walk ×0.9 (own commit)

**The gate** [M disasm 0x1921d4-0x19220c]: GameState @0x2a3774 == 0x07000007 AND
switch_channels[0x48] @0x26fcd8 == 0, then `mul.s f23,f23,f0` with f0 = 0.9 (0x3F666666) — f23
is Player_Move's mult param.

**Which axes it reaches** [M disasm 0x192230/0x192250/0x1922d4/0x1922e0/0x1922c8]: BOTH
horizontal wish lanes — BL+0x10 (strafe) and BL+0x18 (forward·mult) — walk AND swim (the swim
path scales the same two lanes); the turn rate (BL+0x34) is NEVER scaled, and Player_Move has no
stores to the +0x20/+0x28 smoothers, which commit unscaled. The scale applies after the ±0.1
clamps, before position integration.

**The port** (commit 1): `party_slow` on Bond (player.rs:478, own lines), set in scene.rs:3460
beside `jump_locked` (the predicate is literally the same level+channel test), consumed in
`physics_tick`'s mult at player.rs:1656-1657 (`k = (…crouch/swim…) × if party_slow { 0.9 }`).

**The test** `party_move_scale_07000007_live` (crates/nf-view/tests/): real input from the
07000007 spawn, ch 0x48 asserted clear first; the forward wish lane (`Bond.move_local[1]`)
converges to 0.1×0.9 = **0.09** (target_f 0.105 clamps ±0.1 [M 0x1921d0]) and the strafe lane to
0.075×0.9 = **0.0675**. **Red proven on the old code**: with the fix stashed the run fails with
`got 0.1`; with the fix it passes (green both runs, 0.87-0.97 s).

**Merge note (the task's own words):** this changes how walking feels on the party — it will
not merge without a hands-on check. The sim-level evidence is green: the whole
`walkreal_07000007_live` binary passes WITH the scale (6 passed / 0 failed), so the walk pins
tolerate the 10% change — but the feel needs the main agent's run.

## 3. `Player_CollisionHandler` 0x1949a8:97-108 — the 07000012 void-fall backstop (own commit)

**The full condition** [M disasm 0x194b64/0x194c14-0x194c50]: in the handler's airborne arm
(BL+0x940 grounded history == 0 — the shift register, |8 while grounded), after the
fall-timer update (BL+0x928 — a separate arm, NOT a gate for this): GameState == 0x7000012 AND
obj+0x34 (position y) < −10.0 → `switch_channels[0x7C] = 1` (0x194c44 `sb v0,-756(v1)`),
idempotent EVERY tick — no damage gate, no velocity-sign gate, no edge detect (the C's `goto`
is a decompiler join artifact; both arms continue into the same hit-list walk, so the branch
skips no damage processing).

**What reads 0x7C**: the only literal `switch_channels[0x7c]` writer in ACTION.ELF is this
branch; the data side writes the same channel — the plunge plates #788-791 "plunger_sk/1/2/3"
(kind 0xea TouchOnce, p0=0x7c p1=8, at y −10.51 the shaft bottom [D 07000012-audit:1010-1013,
:1156]) and the shaft-fall death NIS SP #767 (trigger ch 0x7C, 110 ticks — writes ch 0x62 = 1,
the death restart [D audit:1132/:1151]). 0x07000011/0x07000012 are adjacent parts of one
mission — the 07000011 doc's #314-318 PlungeTrigger planes share the channel in the same
mission namespace. The engine branch is the backstop that fires FIRST (y < −10.0 vs the plates'
−10.51 plane).

**The port** (commit 2): `void_fall` on Bond set in `physics_tick` right after the
grounded-history/fall-timer mirror (player.rs:2074, own lines; `!swimming && grounded_history ==
0 && pos[1] < −10.0`), and the Sim turns it into the write — scene.rs:3471-3473
`if self.level == 0x0700_0012 && mem::take(&mut self.bond.void_fall) {
self.drones.channels.set(0x7c, 1, self.drones.frame); }` (the level gate lives in the Sim,
which owns the level and the channels — the `jump_locked` injection pattern; `mem::take` clears
a flag left over from a skipped tick).

**Tests**: `the_void_fall_flag_needs_airborne_below_minus_10` (nf-game lib unit test, the
established hand-built-world pattern): falling through an empty world latches the flag only
below −10 while the grounded history is 0, and a floor at y −20 clears it on landing. The
pre-fix port cannot even express this test (the field did not exist — the arm was unported);
that absence IS the red. The whole `walkreal_07000012_live` binary passes with the write in
place (3 passed / 0 failed) — the normal route exits at the shaft entrance before any deep
fall, so the write never fires on it, exactly as in the game.

**What's left for this fix**: the real-fall red/green test on the level (a fall from a ledge
into the shaft, polling 0x7c per tick and asserting the first rise at y > −10.51 — the engine
signature, ahead of the plates' plane). Reaching the shaft on real input needs the objective-1
chain (the Q-Decryptor G-cycle, the SS lift doors slide on ch 0x59) — `tower2::walk_12` +
`walk_12_exit` cover the route; the test did not fit this window.

## 4. Verification summary

- `cargo test -p nf-game --lib`: **685 passed / 0 failed / 5 ignored** (includes the new unit
  test and every existing player test with the two new fields).
- `walkreal_07000007_live` whole: **6 passed / 0 failed** — with the ×0.9 scale live.
- `walkreal_07000012_live` whole: **3 passed / 0 failed** — with the 0x7C write live.
- `party_move_scale_07000007_live`: green with the fix, **red proven** with the fix stashed
  (`got 0.1`).
- The fast crates were green at the branch point (nf-formats disc_golden 11/3 = the allowed
  slim-ISO three; identity_disc = the onfoot sector-count artifact, not a regression); no
  nf-formats/nf-audio code changed since. The other walkreal binaries did not run this window:
  the two fields default false and both new behaviours are level-gated (0x07000007/0x07000012),
  so other levels' walk inputs are unchanged; the two affected levels' binaries ran whole.

## What's left

1. The hands-on check of the party slow-walk feel (the main agent's PCSX2 run on the 07000007
   party route) — the task itself requires it before merge.
2. The level-12 real-fall test (the route spec in §3).
3. The `MenuManager_Monitor` 0x21d508:47 branch — blocked on porting its guarded block (the
   controller-hotplug/attract manager); scope note in §1.

## Live checks for the main agent

1. **The party slow-walk feel** (07000007, before the hall plate #412): walk the party route in
   PCSX2 with the branch — the walk should feel ~10% slower than master in BOTH strafe and
   forward, the turn speed unchanged, and identical again once ch 0x48 sets (after the hall).
2. **The 07000012 shaft fall**: with a debugger watch on switch_channels[0x7C]
   (@0x26fd0c, byte), step off into the shaft on a route where the exit gate #669 does not fire
   (the objectives incomplete): expect the write at y ≈ −10.0 (the engine branch), BEFORE the
   plunge plates' −10.51 plane, then SP #767's ch 0x62 → the death restart back at y ≈ 1.09.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
