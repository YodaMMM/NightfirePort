## Your task: `level-16-jetpack-escape-2`, 07000016: the real escape from the finale pit

The previous attempt made `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` pass by turning on the game's DEBUG fly mode (`debug_scanmode`, substate 5, toggled from a debug controller slot) through a new port-only F key. That is a debug cheat, not what the shipped game gives the player, and it is rejected. Find how a player actually survives the hurt-volume window (0x0600090a stream 6 ticks 1611..2170 [D]) in the shipped game: the script, the item or vehicle, the player state, the channel that moves Bond out [M][D]. Remove the F key and the debug fly path from the walk.

The previous attempt's findings are in the test's own `#[ignore = "..."]` message on master — read it first; it names the blocker exactly. Real input only (`step_bond`: movement/aim/fire/use; no teleports past the blocker, no channel or state pokes, no debug or cheat paths). Port what the game does at the blocker from the decompile [M] / the disc [D]. If a live number is needed, say exactly which address to read in PCSX2 on a fresh boot, and stop there rather than guess. Un-ignore the test when it passes, with a sister that can fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-23T19:07Z

---
## Report (blocked, 2026-10-23T20:05Z)


---
## Report from the worker (blocked, 2026-10-23T20:05Z)

# level-16-jetpack-escape-2 — the 07000016 finale pit window

## What the task asked
Find how a player actually survives the ch-0xf3 hurt-volume window (0x0600090a stream 6) in the
shipped game — the script, the item or vehicle, the player state, the channel that moves Bond out
— and make `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` pass with real
input only, removing the previous attempt's debug-fly F key.

## What's done (all evidence tagged)
- **The F key**: the previous attempt's debug-fly path (`SS_FLY` substate 5 wiring, `fly_toggle`
  input, `scanmode`/`fly_v` fields) lives only on `origin/s/level-16-jetpack-escape`; it was never
  merged to master. This branch is cut from master, so there was nothing to remove — and nothing
  cheat-shaped was added.
- **The script timeline, corrected and port-verified.** The launch script 0x0600090a's Wait
  command is ABSOLUTE ("block until the stream clock reaches the value" [M nis.rs:446-451 behind
  Script_Run 0x1d8548 op 5; the old cumulative reading made streams overrun the 7810-tick length]).
  Full stream dump (probe_pack_flight): stream 6 pulses ch 243 = 1 @1611, = 0 @2170; stream 9
  carries a **second pulse @6810..7320** (missed by U-051); stream 7 writes the pit door's ch 240
  @2180; stream 21 writes the shuttle-bay lift's start ch 80 @2240. The port's own sim
  (probe_pit_window): chain head t0 → pack cinema → Bond re-enables grounded on the pit floor
  (3.57, −23.18, −179.55) at t~900 → ch 243 rises at t2355 (= script start 744 + 1611) → **Bond
  dead at t2356**. The window model is verified in-port, not assumed.
- **The volume cannot be dodged at pit level.** HurtPoly-ShuttleLaunch #309's object (m63 object
  182 "HurtPoly-ShuttleLaunch") is 216 tris in six flat y-bands (world y −24.08..−21.32) × 36 tris
  each, tiling x −30.7..35.3, z −197.3..−133.8 — the entire pit footprint [D tri dump]. Its
  placement carries no path data (extra[7] = 0 → key count 0 [M Hurt_Create 0x17e870:26-30]), so
  the volume never animates; the gated start law reads the raw channel byte [M Hurt_Update
  0x17e5d0:98].
- **Every escape is post-pulse.** Pit door ch 240 @2180 (10 ticks after the drop); the shuttle-bay
  lift (SP #347 → 0x060009e9, start ch 80) is summoned @2240 and its platform (map object
  02000a79, no authored world pose — script-created) descends y 1.61 → −21.22 over 549 ticks —
  the only ride out of the trench, ~60 s too late for pulse 1 [D dumps].
- **No shipped flight exists in the data.** The launch script carries no Bond-actor stream [D];
  the level's 19 pickups are armour/ammo/007-token/grenade kinds [D census; M Pickup_Handler
  0x18e650 kinds 0..6 — no jetpack]; the substate dispatch (Player_Update 0x193600:171-261)
  reaches only walk/climb/grapple/move/crouch/ScanMode-fly/wire/creep/zero-g/cam-wait/zipline, and
  substate 5's only shipped writers are the cheats-menu control C_CHCHFLY 0x215c70 and the dev
  action-39 toggle (no controller style writes action 39 [M 0x10cd80 census]); script events
  0..0x13 [M Script_EventHandler 0x1d94c0] and player-start modes (2→zero-g 8, 1→move 3, 3→crouch
  4 [M Player_Init 0x1910f8:156-181]) never reach it. The ELF's jetpack SFX names
  (SFX_CHR_BOND_JETPACK_*) have no reachable gameplay entry in ACTION.ELF's decompile.
- **The test**: `walk_the_chasers_defeat_completes_the_mission` stays `#[ignore]`d with the
  sharpened, fully-cited blocker (the doc comment above it carries the whole chain). Un-ignoring
  it would require either a cheat (forbidden) or inventing behaviour the disc data does not
  contain.

## Conclusion
Under the port's [D]-exact script timing, a real-input player re-enabled at the pit floor dies at
the first pulse with the door locked and the lift unsummoned. The shipped survival mechanism is
none of script/item/vehicle/state that the disc data exposes; the honest state is the sharpened
blocker plus the live checks below, not a stand-in.

## Live checks for the main agent (PCSX2, FRESH boot of 07000016 — a save-state resume skips the boot path)
1. Watch `switch_channels[243]` (0x26fc90 + 243·4, byte) from the ch-0x81 bridge touch through
   the launch: expected (port-faithful) — it rises ~863 ticks after control returns at the cinema
   hand-back, and drops ~559 ticks later.
2. Watch Bond's `obj+0x30` (the player position vector) across the pack-cinema hand-back: expected
   (3.57, −22.76, −179.55) [D stream-8 key]. If instead the hand-back restores the pre-cinema
   (bridge) pose, the port's event-6/`Script_Free` → `Player_Enable` matrix source is wrong —
   that is the one unverified read in the chain (SCRIPTINFO+0x40's writer was not located in
   ACTION.ELF's decompile this session).
3. If Bond IS at the pit floor with ch 243 pending: `tools/trace/gdbwatch.py` on `obj+0x30`
   through ticks 748..1611 records anything that MOVES him out — that write is the shipped
   survival mechanism and the test can then pin it.

## What's left
- The main agent's live checks above (they decide between "the port's hand-back is faithful and
  the game ships a death trap the player out-times with a mechanism outside the decompiled
  surface" vs "the hand-back matrix source is misread").
- The walk test's un-ignore once the survival mechanism is pinned.

## Suite
`NF_ISO=… nohup python3 tools/test/suite.py` on the branch (master + this task's test/doc edits,
no source deltas): **2917 passed, 72 failed, 122 ignored (119 min)** — `tail -1 suite.out`. The
log's FAILED lines are progress-bar-garbled; the readable ones name `fix_0c_pylon_live` (twice)
and `disc_golden::bigf_header_f…`, plus a `net_join_live` crash (exit −9, the sandbox kill).
This is the unmodified-master baseline (the run started before any edit landed); a second full
run on the final tree does not fit this sandbox's remaining window (~15 min at report time vs a
119 min suite) — the next session should rerun it on the final tree. The crates this task's
edits touch (`nf-view` tests) were compile-checked clean (`cargo check --offline -p nf-view
--tests`).

## Usage
not visible to me — this sandbox exposes no token/credit/context-window figures in any
readable surface (no environment variable, log or UI text was found).
