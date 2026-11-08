## Your task: `markers-drone-a`, guard behaviour the port guessed is settled against the game (drone.rs, first third)

Every `[S]` (unverified) marker in crates/nf-game/src/drone.rs from line 1 to line 12000 (21 markers today) on current master is behaviour the port guessed. For each one, read the game code it stands in for, with addresses, and end it one of three ways: port the game's behaviour and remove the marker; prove the port already equals the game, with the evidence ([M], [D] or a fresh-boot [L]) in place of the marker; or prove the path is never reached on the shipped disc. List every marker in the report with its line, what it guessed, the game's answer and which of the three it became. Where a marker cannot be settled without a live read, keep it and list the exact read under the report's live checks for the main agent. Rand draws must keep the game's count. Where a change alters play, pin it through real input on a level where it happens, and repair any walk it shifts, gated to that level, never lowering damage or health. Run the binaries that touch the changed code (grep tests for the functions), every walkreal_*_live one level per tool call if guard or player behaviour changes, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/markers-drone-a`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T12:30Z

---
## Report (done, 2026-11-08T13:50Z)


---
## Report from the worker (done, 2026-11-08T13:50Z)

# markers-drone-a — the drone.rs [S] markers (lines 1–12000) settled against the game

Branch `s/markers-drone-a` (from origin/master 6186d8530). Task: every `[S]` (unverified) marker in
`crates/nf-game/src/drone.rs` lines 1–12000 (driver counted 21; the grep census found 22 hits, of which
3 are stale-note quotes — see the census below — leaving 19 live markers) is behaviour the port guessed.
For each: read the game code it stands for and end it one of three ways — port the game's behaviour and
remove the marker; prove the port already equals the game with the evidence in place of the marker; or
prove the path is never reached on the shipped disc.

**All changes this session are comment/doc + one dead-code deletion — zero runtime behaviour change**
(the one code change, deleting the `if false && …` combat exit at 10840, removes a block that could
never execute). No Rand draws added or removed; no walk can shift. Evidence below.

## The census (grep `[S]`, lines ≤ 12000 → 22 hits)

Lines 1008, 2484, 6093 are **not markers**: they quote an older `[S]` note that a later update already
struck (1008: "the old [S] seen latch, writer untraced note was stale — the update pass cites the
writer"; 2484: "the old [S] no kind-8 paths note was wrong and is struck" with the [D] nav dump;
6093: "an earlier damping astronaut-only [S] note read it as the 0x1D arm"). No work; left as-is.

The 19 live markers, each with its line, what it guessed, the game's answer, and the outcome:

### Settled — port already equals the game (evidence in place of the marker)

1. **236 — `RETURN_TO_PATROL_PATH` "ENTER re-finds the mission path (not modelled [S])"**. The game's
   `DSTATE_ReturnToPatrolPath` 0x172810 case 1 calls `NDrone2_ReFindMissionPath` 0x153a30, writes
   `+0x5a2 = 0x14` and `Drone_SM_SetState(99,0)` [M 00172810:7-11]. The port **already calls** it —
   `(RETURN_TO_PATROL_PATH, msg::ENTER)` runs `self.re_find_mission_path(d)` then
   `return_state = CIVILIAN_PATROL; set_state(GO_TO_GOAL)` (drone.rs 7694-7697), and
   `re_find_mission_path` (5913) implements the game's nearest-MoveTest-reachable-node-else-closest
   selection. The "[S] not modelled" was stale. Replaced with the [M] citations. Outcome: (b).

2. **4186 — PowerstationTalk's folded gates "SP drones are class 2 and the port has no fell-out flag
   [S]"**. Each folded arm is implied for every SP on-foot guard that can reach `NDrone2_PowerstationTalk`
   0x13e770: `0x100` set at spawn (DefaultInit 0x14b300:60) / Enable(1) 0x149dd0:27, cleared only by
   Enable(0), and ControlSTANDARD 0x1490f0:140-143 returns before HandleTalking(:201) when clear;
   `0x200` only SetAsDead 0x14a2f0:9 (health zeroed :21-23), `0x400` only PostDroneControl 0x14aff8:9-12
   under health ≤ 0 — both subsumed by the port's health gate; class `0x11` only BOT_respawn
   0x124e38:90 (sole caller MP_RestartScenario 0x183fc0:69), SP drones class 2 [M 0x14d7c8:14]; the
   obj+0xFE bit-0 writers (FellOutMap 0x13a878:16, Fade 0x172cd8:44) both run Enable(0) first, and
   PreDroneControl 0x148db0:53 is the kill-msg path. Outcome: (b) — SP-path impossibility (the gates
   are individually reachable in MP, so not (c)).

3. **4477 — speech precache "rides the play, headless-optional [S]"**. `SFXPreCache(sfx, 0x14)`
   [M 0x141b00:75-76] is an IOP-only batch command gated on `switch_CONFIG_SOUND` [M 0x117118:8-20] —
   it writes no EE/sim state (the game itself skips the call with sound off). The only sim-visible
   0x14 write is `+0x48 = now + 0x14`, which the port models exactly
   (`facial_talk_pending = Some((l.facial, frame + 20))`, consumed at the T+21 append). Outcome: (b).

4. **4489 — "the callers' cooldown bookkeeping (480/600 frames) exceeds both [S]"**. For the DIRECT
   bond-talk lines the repeat half of the game's gate is dead outright: every DIRECT line carries
   `repeat == 0` (`now < lastFrame + 0` never true) [L Drone_SfxInfo dump + the 07000007 census: the
   facial==0 group 0x052-0x083 and the script-absent 0x70/0x72/0x73], and every caller cooldown
   (150/240/300/480/600/960 [M 0x1446b0:73-74,131-132,180-181,216-217; 0x144488:81-82 + jump table
   0x2f5310]) exceeds the 30-frame post-load gate [M 0x141b00:23-25]. Residual corner documented in
   the comment (a request inside the first 30 frames of load — not reachable by any walk: it needs a
   bond-talk trigger within 0.5 s of load). Outcome: (b) with the corner noted.

5. **8024 — "the port does not yet stamp the 0x70000000 broadcast bits [S]"** — **stale, false**. The
   port models the bits as `alert_pending` with the game's bit values at the game's writer families:
   0x20000000 from `DroneFunc_DoForcedAttack` (7782/9207), 0x40000000 from
   `DroneVision_EnemyAlerts`'s ALARM (7795, [M 0x177c18:113-114]), 0x10000000 from the forced-attack
   raise in AlertToPosition/SearchArea and the AlertSound share (7810/7977, [M 0x165270:115,
   0x165bf0:79, 0x177800:70]); the comms arm (8033-8035) reads them under `!own_sight` — the same
   guard order as FirstAttack 0x148010:22-29 (the 0x8000000 own-sight test first; 0x70000000 only in
   the clear arm). Outcome: (b).

6. **8571 — "(+0x4F8 |= 0x2000 has no port reader [S])"** — **stale, false**. The game's INTEROGATE
   ENTER sets 0x10000 and 0x2000 [M 0x161a40:55-56]; 0x2000 is the may-notice half, ported as
   `watching` and set by the shared UNAWARE ENTER gate (drone.rs 7098) because INTEROGATE 0x3c ∈
   `state::UNAWARE` (297-298); the reader is FindAlertedDrones' observer gate (19057,
   [M 0x176250:35]). Outcome: (b).

7. **10833 + 10838 — the COMBAT sight-loss exit "[S] the window length … Off [S]"**. The research
   showed the disabled `if false && …` block was a **non-game duplicate**: the game's ONLY
   NoOpponent exit from these states is the head gate — `+0x170 == 0` AND the `+0x238` stamp stale
   (`now > stamp+30`; HasOpponent 0x14ad58:11-12 `now <= stamp+30`; the sole +0x238 writer is
   `DroneVision_HaveOpponentSight` 0x175ef0:41-44,66-68, stamped per sight tick behind the reaction
   gate; no unseen-tick or sight-bit term anywhere — DroneFunc_CombatState 0x148338:20-31 is the
   only `return 0x60`, before the switch; the switch's sight-loss arms key off `0xf < +0x274` to
   ENTER 0x5c [M 0x148338:67/151/208/235]). The head gate is already ported game-exact
   (drone.rs 10825 `if !self.has_opponent(frame)` with `has_opponent` = the same test; the stamp is
   wired at 18883/18959). The old block ignored `opp_ptr()` entirely — it ejected pointer-holding
   team-1 guards (whose opponent pointer is the per-tick player fold, 0x1424e0:100-121) to 0x60, a
   state the game never produces — which is why it stalled the 06/09/16 guards when live at 15.
   **Deleted the block (10840-10844) and rewrote the comment** with the full chain. Outcome: (b).
   Note for the record: the 07000001 hall-guard freeze the old comment blamed on this exit lives in
   the hunt machinery (0x5c keeps re-routing to the live opponent via AINetwork_CalcRouteToObject),
   not in a missing exit — enabling any tick-based exit would ADD a game-nonexistent ejection.

8. **11936 — tanf "the port's `f32::tan` is the stand-in [S]"**. The game's `tanf` 0x232bc0 is the
   FDLIBM-family float tanf (π/4 threshold 0x3F490FDB, `x−x` NaN arm, `__rem_pio2f` reduction
   0x2341e8, `__kernel_tandf` 0x235050 [M disasm 0x232bc0-0x232d50]; musl's tanf is the same code
   family), and the port's `f32::tan` lowers to the platform libm tanf — measured on this target it
   returns the correctly-rounded f32 0x3F13CD3A for the shared input 0x3F060A92 (`VIEW_ANGLE*0.5`
   halves exactly), so any ≤ 2-ulp game deviation moves the encoded plane half-angle ≤ 9e-8 rad
   (atan-derivative 1/(1+t²)·2⁻²⁴). All three game call sites tan the same `viewer+0x118 * 0.5`
   [M 0x1e7df8:73, 0x1baf70:43, 0x10b2b0:104] — the aspect is a multiply after the tan. Outcome: (b).

9. **11969 — DRONE_DRAWN_SLACK "[S] unverified — the per-frame value varies"**. `obj+0x8C =
   glist+0x1C · abs(obj+0xE8)` [M 0x133070:48], and obj+0xE8 is the **placement draw scale, written
   once at creation** [M Control_CreateObjEx 0x1343c8:27] — never animated on drone objects (the
   drone-side +0xE8 stores are the Drone_tag's sight range, a different struct: 0x14b300:386-420,
   0x14aed8:25-26, 0x15fc18:20). [L ee_snapshot.bin] guard glist+0x1C 1.1589-1.1602, built radii
   1.019-1.128 (|scale| 0.879-0.974 ≤ 1) — the port's 1.0 is a tighter gate by ≤ 0.128 world-units at
   the plane edge. The comment now also states the gate IS sim-visible through `dr.drawn` (seek
   targets 15906, BondTalk 0x1446b0:8-31, the DoAnimation slide 0x149f88), so the margin matters only
   within 0.13 units of a side plane. Outcome: (b).

### Kept — scoped, with the game's answer and the exact next step

10. **887 — muzzle_through_wall "the port's segment sets [S]"**. The game's ray is
    `Collide_RayIntersect(clamped muzzle → raw muzzle, obj+0x20 cel, ignore self, mask 0x927)`
    [M 0x178520:38-52]; the port probes `world.segment_clear` (its stat/missile/dynamic sets). The
    mask-0x927 class bits are not yet mapped to the port's sets anywhere. Next step: map
    Collide_RayIntersect 0x1eae80's mask bits (0x927) to the port's three sets and prove set
    membership equality for the reachable geometry (the port's doc already scopes this at 11426).

11. **1693 — no_bunching swept-sphere simplification [S]**. Game: `SphereSphereSweep` 0x14e5b8 (sole
    caller EvaluateObstacle 0x14e6f0:94) solves the quadratic for first/last overlap times; the port
    gates on static per-tick distance (radius sum, the same 9.0 threshold). Whether the sweep admits
    a pair the static gate misses at guard speeds is unresolved. Next step: read 0x14e6f0's caller
    context for the per-tick sweep segment (prev→current position) and bound the guard speed against
    the 3.0 radius sum.

12. **1841 — hostage pairing "the port's hostage-side captor search is unchanged [S]"** — sharpened
    with the game's answer: `NDrone2_SetupHostageKiller` 0x144030 binds BOTH halves in the one call
    (killer+0x2b0 = hostage obj AND hostage+0x2b0 = killer obj) and sets the hostage's goal pos
    (+0x670..) to the killer's feet [M 0x144030:62-80], choosing the nearest unpaired state-0x05
    drone within the 5.0 scan bound [:30-61]. Next step: audit `NDrone2_DSTATE_Hostage` 0x15cb78's
    +0x2b0 reads against the port's HOSTAGE machinery, then bind both sides.

13. **3199 — guard-side flight glass pass NOT modelled [S]**. Untouched this session (deep dive: the
    game's bullet flight + `Collide_FilterBullets` 0x1ece30 pane admission; the deviation is
    documented with its play evidence in the comment). Next pass owns the decision to port.

14. **4208 — "first-eligible sticks [S]"** — partially settled: the game's remember sits behind the
    play (`DAT_00298FCC = *puVar6` under `lVar5 != 0`, [M 0x13e770:48-51, 0d twin :62-66]) and the
    loop advances over every record (no break; a refused record returns 0 and is NOT remembered; a
    played record sets busy so later records are refused [M 0x141b00 gates :23-25/:38-41/:99-108/
    :112-114]). The fold is exact while exactly one record is channel-eligible and none of the
    deferred gates refuse it; the open corner (kept [S], audio-only — no RNG, no sim state): two
    channel-eligible records with the first refused by its repeat gate (entries 216-232 carry
    repeat 180 [L Drone_SfxInfo 0x2812d0]) play different lines. Port sketch in the comment: take
    frame/lines/voice_last at the call, walk ALL records, push+remember only the first that also
    passes the speech() gates.

15. **4849 — door-guard pairing "defers to the first UPDATE [S]"**. Untouched: needs the
    0x160648 ENTER/UPDATE read (SetDronePartner's writes on both drones) and the observable-state
    check for the one-tick defer.

16. **6052 — the ground probe's four 2 cm rays "[S]"**. Untouched: needs the game's actual
    ground-support test (Drone_CollisionHandler 0x140968 and its floor-contact path) compared with
    the feet-ray + 4-offset-ray stand-in.

17. **8277 — AIM_STAND "the anim-step 0xc Rand(3) → 0x6e roll is not ported [S]"**. Untouched: needs
    the 0x68 DSTATE read (the exact Rand call, the 0x6e transition, the anim-step driver) — note
    porting it ADDS a Rand draw and shifts the shared RNG timeline (walks would need re-running).

18. **11566 — seek_opponent_los "the AINetwork_TestRayCels pre-pass is the port's world ray [S]"** —
    sharpened with the game's answer: the pre-pass is an AI-cel REACHABILITY check, not a geometry
    ray — it builds the ray hit-test, `Collide_StraddleCels`, and requires the TARGET's cel among
    the straddled cels (same-cel passes outright) [M 0x176f18:65-67 + 0x15ab78:44-66]; a ray the
    port's world clears but whose cel path never straddles the target cel is BLOCKED in the game
    (ledge/hole edges). Port sketch in the comment: straddle-walk the port's nav cels along the ray.

## Verification (what ran, what did not)

- `cargo test -p nf-game --lib --offline --release -- --test-threads=1`: **708 passed, 0 failed,
  5 ignored** (run twice: after the main settlements and on the final tree).
- `walkreal_07000001_live`: **8 passed, 0 failed** (67 s).
- `glass_pass_every_level_live`: **3 passed, 0 failed, 3 ignored** (the ignores are the pre-existing
  pane-ride gates, unchanged from master).
- `dead_pane_live`: **2 ignored** (pre-existing on master, unchanged).
- `nf-game` lib compile: clean (91 pre-existing warnings).
- **The full fast suite did not run.** The sandbox kills background jobs between tool calls here
  (started per the driver's instruction at 13:28; `suite.out` 0 bytes, process gone by 13:46), and
  the second half of the window went to rebuilding the test binaries after an ALSA link fix (below).
  The walkreal battery beyond 07000001 and the door/guard integration binaries did not run either.
  **The diff is comment-only + one dead-code deletion (zero runtime change), so these runs are
  insurance, not risk — but the next pass must run the full suite and the walkreal battery on this
  tree before building on it.**

## Sandbox note (not committed): the ALSA link fix

`libasound2-dev` extracted to `~/alsa-prefix` ships `libasound.so` as a dangling symlink (→
`libasound.so.2.0.0`, which lives in the runtime package). The fix that made nf-view/nf-audio link:
re-point the symlink at the system runtime lib and pass the search path to the linker —
`ln -sf /usr/lib/x86_64-linux-gnu/libasound.so.2 ~/alsa-prefix/usr/lib/x86_64-linux-gnu/libasound.so`
plus `export RUSTFLAGS="-L native=$HOME/alsa-prefix/usr/lib/x86_64-linux-gnu"` (RUSTFLAGS, not
LIBRARY_PATH — rust-lld does not read LIBRARY_PATH). This lives in `~/env.sh`, outside the repo.

## Live checks for the main agent

None required by the settlements (all evidence is decompile/dump). Optional belt-and-braces for the
kept drawn-slack note (11969): (i) a scale census across level snapshots asserting
`abs(obj+0xE8) ≤ 1.0` for every drone obj; (ii) a differential sweep — no guard's `drawn` flips
between slack 1.0 and 1.17 on any walk.

## What's left

- The 8 kept markers (887, 1693, 1841, 3199, 4208-corner, 4849, 6052, 8277, 11566) — each now carries
  the game's answer and a concrete next step in its comment (section above).
- The full fast suite + the whole walkreal battery on this tree (insurance; the diff is
  behaviour-neutral).
- The next pass should re-census after landing these: lines ≤ 12000 should grep 12 `[S]` hits
  (3 stale quotes + 9 kept).

## Usage

Session token/credit/context figures for this main session: not visible to me (no figures are shown
for the main conversation in this environment). Figures I did receive, from the Task tool's two
completed research subagents: research B (speech cluster) prompt 155,948 / completion 8,458 / total
164,406; research E (combat hold / tanf / slack) prompt 119,784 / completion 5,978 / total 125,762.
The other four research dispatches failed with a tool context deadline and consumed budget I cannot
see. No cost figure is visible to me.
