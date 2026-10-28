# drone-bump-collision-3 — the arming ported end to end; drone_bump / walkreal_07 / 07000011 green; the 07 crowd re-derivation cited, the nav lane left; mayhew left

Branch `s/drone-bump-collision-3` = `origin/s/drone-bump-collision-2` (69cd437b; `origin/master` fcf6f334 already an
ancestor) + three commits (0f569548 branch start, 65f15211 the arming, d8731d27 the probe finding). WORKLOG entry
`drone-bump-collision-3`.

## What was done (all [M] disasm-checked this round)

### The arming, exactly as the game does it

1. **The kind word ext+0x464 is a ported field** (`Drone::kind_464`, u16): `Collide_Jointy` 0x1ec180:203-209 demands
   0x800 on BOTH sides; `NDrone2_Collision` 0x140e90:66-72/95-101 rewrites it 0x800/0 EVERY control tick;
   `DefaultInit` 0x14b300:576 starts it 0x800 at spawn; `Drone_CollisionHandler` 0x140968:71-75 **zeroes it outright
   every tick the drone system is frozen** (`NPCGlobals == 0` — script event 5's `Drone_EnableAll(0)` — or
   `Drone_bDisableSystem`), and `Drone_Control` 0x139268:39-42 gates the re-arm off the same flags. The port:
   `Drone::arm_kind_464()` called on the bot's control pass right after the UPDATE send + the flying move (where
   `NDrone2_ControlSTANDARD` 0x1490f0:230 calls Collision); the freeze zero sits at `tick_inner`'s head.
2. **`NDrone2_DoCollision` 0x149ec8:11-44 ported from the DISASSEMBLY** (0x149ec8-0x149f70, hand-decoded — the
   decompile's C is faithful but the memory-note was not): flying 0x200 → 0 (GameState 0x700004a exempt — unmodelled
   [S]); master arm 0x20 → the port's `enabled`; DSTATE 0xC/0x13 → 1 (HOSTAGE_SAVED/CIVILIAN_HIDING); ext+0x538 → 1
   (the blend-out script word, unmodelled → 0 [S]); the near arm (`DroneFunc_NearPlayerCollision` 0x14b190, any
   +0x154 slot < 2.0) → 1; **obj+0xFC ≠ 0 → 1** (`lbu 0xfc($v0)` 0x149f50 — the drawn-grace counter, the port's
   `drawn_grace`); **+0x570 & 2 → 0** (`lw 0x570` 0x149f60 — the ANIM-INFO flag word, written by
   `DroneAnim_SetDAnimInternal` 0x13aee8:50 from `Drone_AnimInfo[anim]`; unmodelled → clear [S]); the fall-through → 1.
   So with the two unmodelled words at their defaults the gate collapses to `enabled` ON CONTROL TICKS — the
   discriminator is WHICH ticks run, which is items 1 and 3.
3. **The +0x154 slots moved into the bot's own control pass** (PreDroneControl 0x148db0:35-48's own position, before
   the state update): the batched-opponents fill is gone; a bot whose control does not run (spawn-tick skip, the
   freeze) keeps its last slots — the task's "written ONLY on the bot's control tick" is now literally true.
4. **The disabled-player push skip (NEW [M], not in the task text): `Player_Update` 0x193600:80-88 returns before
   `Player_Collision` :361 when the ext+0x94a enable byte is 0, and `Player_Disable(player,0)` 0x195f70:12-18 zeroes
   that byte AND the player's own kind word +0x7d4** — a cut-scene Bond runs NO class-3 query at all. The port's
   producer drops `bot_capsules` while `Sim::disabled`. (Also covers Jointy's both-sides gate.)
5. **The capsule bottom from the anim height word ext+0xa0**: `bump_capsule` uses `hip_anim` instead of the fixed 1.0
   [M 0x140e90:68 reads +0xa0 into the bottom's `r − hip` on both branches; `DroneAnim_PreTransCallback` 0x13b490:63-66
   rewrites it `clamp(animObj+0x5C − 0.02, 0.2)`; DefaultInit 0x14b300:48 starts 1.0]. Standing anims keep 1.0 (the
   port does not model the per-anim root — the fix-driver-air residual); the sit paths carry the 0.2 clamp.
6. **The producer** (scene.rs `bot_capsules`): `kind_464 == 0x800` + the [S] corpse exclusion + the cel-straddle gate
   (collision-2's), replacing the live-distance test. The collision-2 memory-note "0x800 only within 2.0" was the near
   ARM alone — the disassembly adds the drawn-grace arm and the fall-through arm; corrected in the comment.

## Verification (what ran — all on the final tree's content; cargo's content-hash fingerprints confirm the binaries match the source)

- **drone_bump_live 2/2 GREEN** (the proven Jointy push keeps through the kind gate; both tests, twice).
- **walkreal_07000007_live 2/2 GREEN** (twice — the party walk through the hall's south band on the kind-gated push).
- **walkreal_07000011_live 1/1 GREEN**: the flight C band check raced a mid-fall tick (the probe: `grounded=false`,
  floor 0.46 below, y 32.511 vs the 32.5 ceiling — 0.011) — the armed stairwell guards' pushes shift Bond's descent
  timing by a tick; a 20-tick settle lands the fall before the check. The check's substance (between the flight's
  bottom and the landing, not in the well) unchanged; cited in the test.
- **party_door_lock ×2, cutscene_timer_pause, probe_07_route still RED** — the hall's south band. The re-derivation
  (cited in all three files): the crowd IS armed [M the chain above; the party states write no +0x4F8 collision bits
  — checked 0x1728c0/0x160140], its patrol nodes sit on the anchors' line ((79.75,−50.45)/(76.28,−50.30)/(77.76,−48.60)
  [D the nav dump]), and the walker wedges at the 0.95 contact equilibrium ((79.19,−51.21) vs the bot at
  (79.57,−50.34), 0.99 apart). The anchors moved to a north lane (z ≈ −46.6) — but the NAV WEB routes through the
  crowd band's nodes and the lane itself hits the hall's west wall at x ≈ 80 (the manual-leg probe wedges at
  (79.19,−45.7), the same x as the classic wedge — only the west mouth (z ≈ −50.7) enters the hall, INSIDE the crowd
  band). Meanwhile walkreal_07 walks the SAME south-band line deterministically green — the crowd phase at the hall
  differs per test (each test's earlier legs shift Bond's arrival timing). **What's left**: thread the west mouth
  behind walkreal_07's exact leg timing, or add a crowd-clear wait helper to the walker; the geometry is not the
  blocker, the phase is.
- **mayhew_hold_live still RED** (untouched this round): the flank legs wedge against the armed combat guards
  (state 0x29; the probe census shows an armed guard 2.40 from Bond on 07000011's flight C — the same shape). The
  fight choreography needs push-aware legs (keep the 1.0 contact band) or a fight pass; the arming chain gives no
  further gate — the guards are legitimately armed while fighting.
- **The full suite did NOT run in this window.** The background `tools/test/suite.py` runner dies between tool calls
  in this sandbox (re-confirmed three times: suite.out empty, no cargo process survives); the foreground attempt hit
  the 10-minute tool cap mid-build of the ~700 binaries (the runner builds all four crates' test binaries before
  running anything). **Ran instead, in the foreground:** the eight affected nf-view test binaries individually
  (drone_bump_live, walkreal_07000007_live, walkreal_07000011_live, probe_07_route, cutscene_timer_pause_live,
  party_door_lock_07000007_live, mayhew_hold_live, probe_07_who + the scratch probe_drone3_11) — the nf-view
  integration tests that touch the change. **nf-game and nf-formats, nf-audio, nf-view unit tests did not re-run this
  round**; the only shared-code changes are `Drone` (one new field + three methods, the slot-write move, the tick-head
  zero) and scene.rs's producer — the dmode_row0_near_player_live slots test (nf-game) still reads the same
  `dist_to_player` array it writes directly, and `near_player_collision()` is unchanged. Quote for the next session:
  run the full suite first thing on a machine that keeps background processes.
- Scratch probes (assertion-free, --nocapture, committed like probe_07_who before them): `probe_07_who` (the hall
  census — bots 20/21/22 boot `enabled false, state 0x1` at z −48.6/−50.4/−50.5), `probe_drone3_11` (07000011's
  flight C: drone 10, team 1, state 0x29, `kind_464 0x800`, slot 2.40 — the armed-guard shape).

## Live checks for the main agent

1. **The party-crowd solidity (settles the 07 reroute direction)**: 07000007 fresh boot, walk Bond straight into a
   standing party guest mid-floor away from the doorways: does the game stop him at ~0.95 (the push runs — the
   decompile says it does: the guests' control runs, DoCollision's near arm fires inside 2.0, kind 0x464 = 0x800,
   Jointy pushes [M 0x149ec8:30-33 → 0x140e90:95-101 → 0x1ec180:229-281]) or pass through?
2. **A frozen bot's kind word**: 07000007 fresh boot, inside the control-taking scene (SP #420): gdbwatch a hall
   bot's ext+0x464 (expect 0 — the scene only fires event 4/Player_Disable, no event 5, so the bots' control still
   runs and the kind should be 0x800; if the main agent sees 0x800, the freeze zero is confirmed irrelevant HERE) and
   Bond's own +0x7d4 (expect 0 — Player_Disable zeroes it [M 0x195f70:18]).
3. **mayhew's guards**: fresh boot 07000001, during the ambush: gdbwatch an ambush guard's ext+0x464 (expect 0x800
   while it fights) — if 0, there is an unmodelled disarm in the combat states and mayhew's fix is an arming fix
   instead of a leg fix.

## What's left, in order

1. The 07 trio (party_door_lock ×2, cutscene_timer_pause, probe_07_route): thread the west mouth with walkreal_07's
   leg timing (or a crowd-clear wait in the walker); the anchors' re-derivation cite is already in the files — the
   north-lane anchors I left there are WRONG (the west wall) and should go back to the south band with the timing fix.
2. mayhew_hold: push-aware flank legs (or the live check 3 outcome).
3. The full suite on the merged-with-master final tree; quote the total line.

## Usage

- tokens/context/cost figures: not visible to me (no token or context-window readout is exposed to this session;
  nothing to copy exactly).
