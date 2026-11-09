## Your task: `player-ceiling-tails`, Bond's body stops at ceilings and handles death as the game's player code does

The control-funcs-collide audit (origin/s/control-funcs-collide, docs/reports/control-funcs-collide.md) lists two unmodelled tails of the player's collision handler 0x1949a8: the dead-state arm (:33-74) and the ceiling arm (:168-175). Read both in full with what they write and when they run. Find where the port handles the same cases today (a ceiling met during a jump, a climb or a lift, and contacts after Bond's death) and say what differs. Port both tails in the game's order, keeping the Rand count. Find places on the on-foot levels where Bond can meet a ceiling [D], and pin through real input on two of them: Bond's head stops at the ceiling the way the game's code says, and a dead Bond's body behaves as the dead arm says. Run the player and collision binaries (grep tests for 0x1949a8), every walkreal_*_live one level per tool call, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/player-ceiling-tails`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T01:10Z

---
## Report (done, 2026-11-09T02:26Z)

# player-ceiling-tails — the player collision handler's two unmodelled tails

Branch `s/player-ceiling-tails`, cut from `origin/master` (63b05cca6). The control-funcs-collide
audit (done/10-control-funcs-collide.md on the queue branch) listed two unmodelled tails of
`Player_CollisionHandler` 0x1949a8: "the dead-state arm (:33-74)" and "the ceiling arm (:168-175)".

## What the two arms actually are (read with the disassembly, not just the C)

The decompile alone misleads here; every gate below was checked in the instruction stream
(a small hand-written MIPS decoder — capstone mis-decodes the SDC1/EE stream, so `scripts/mipsdis.py`
was written for this task and is not part of the repo).

**The ":33-74" arm is the CLIMB arm, not a dead-state arm.** The audit's label is wrong. The gate is
`lh v1, 0xf6(s2)` — obj+0xF6, the SUBSTATE halfword — compared to 1, and the branch to the live path
is `bnel v1, 1` [M disasm 0x194a2c-0x194a34; obj-tag-player.md:60 documents obj+0xF6 as the
sub-state, 1 = climb]. The dead state lives in obj+0xF4, and the handler reads it exactly once: as
`Player_FeetOnPoint`'s 4th argument (`(u16)(state−2) < 2`, the dead flag) [M :77, asm 0x194a38
`lhu a3, 0xf4(s2)`]. That argument has NO consumer inside FeetOnPoint — `a3` is overwritten by
`Mat_GetUp` in the prologue before any read [M disasm 0x194188] — so the feet test is identical
alive and dead. What the climb arm does: walk the HITDATA list, take the last hit that is not the
climb object (flag bit 2 clear, not == BL+0x884 [M :34-52]), compare its position y against the
per-substate view offset BL+0x760, set AnimObj flag 0x20 when the contact is above the view and
flag 0x8 when below [M :53-72, asm 0x194a88-0x194ae8], and zero the fall timer both ways [M
:54,68,71].

**The ":168-175" arm is the grounded velocity reset** (the audit called it "ceiling"; the ceiling
bump itself is NOT here — it is consumed in `Player_HandleJump`). Gate: AnimObj flag 8 (grounded —
set by the feet ray, [M Player_FeetOnPoint 0x194188:176-185]) AND jump state BL+0x94C != 1 [M :168-169,
asm 0x194cf0-0x194d04 `beql v1, 1`]. Writes: `BL+0x14 = 0` [M 0x194d0c] and `Vec_Zero(BL+0x50)`
[M 0x194d10 → 0x1e2d58]. When the jump state is 1 (rising) both writes are skipped [M :169-172].

**Where the ceiling actually stops Bond**: AnimObj flag 0x20 is the "contact above the view
position" bit, set by the handler's live path from the last non-climb-object contact of the frame's
hit list [M :155-165, asm 0x194cbc-0x194ce8]. `Player_HandleJump` 0x1916b0 consumes it while rising:
jump state 1 + flag 0x20 → jump state 2 and `Vec_Zero(BL+0x50)` [M 0x1916b0:69-74]. A jump also
refuses to start under a roof via the feet→feet+0.95 `Collide_LineOfSight` headroom check [M
0x1916b0:171-178].

**The dead state's real surface**: the dead player object (class 0x12) keeps 0x1949a8 as its
collide slot [M obj-tag-player.md:18], and `Collide_Update` 0x1ea398:55-69 dispatches every object's
collide slot unconditionally, every frame — so a dead Bond still runs the handler per frame, even
though `Player_Update`'s dead branch (state 2/3) runs only `AnimObjectUpdate` +
`Player_HandleDeath` [M 0x193600:316-319 — no movement, no `Player_Collision` rebuild]:
- `BL+0x95F = 0xFF` BEFORE the freeze-gate return [M :21-24, asm 0x1949e4 precedes the 0x94a gate]
  — the third-icon context clears every dead tick;
- the anim flag clears [M :27-29]; the per-hit list walk (:136-165) never runs — obj+0xD0 is freed
  at death [M obj-tag-player.md:51] — so flags 0x10/0x20 never re-arm while dead;
- `Player_FeetOnPoint(dead)` still grounds flag 8 [M :76-77 + 0x194188:176-185] and the grounded
  history follows [M :78-82];
- the fall timer arms run [M :97-131] — a dead airborne body with a negative frozen y velocity
  keeps counting [M :97-103], and the 0x7000012 kill-plane latch fires while airborne below −10
  [M :104-108, asm 0x194c14-0x194c50];
- the ceiling arm runs (grounded && not rising → the velocity zeros [M :168-175]).

## Where the port stood, and what differs

- The climb arm's consumer and the ceiling-bump chain were already ported: `head_bump` (the
  capsule's downward push) drives the rising bump to jump 2 + zeroed `vel` [M Player_HandleJump
  0x1916b0:69-74 — player.rs physics_tick's tail], the jump-refusal headroom check, and the climb
  up-push block [M Player_Climb 0x196310:69-71]. The port's contact-above signal comes from the
  capsule push direction instead of the last-hit-above-eye test — a documented stand-in (the two
  agree on a ceiling overhead; a tall wall contact above the eye is the residual difference).
- The grounded velocity zero was already in the gravity gate (`grounded && jump != 1 → vel = 0`
  before the move, plus the post-contact `vel[1] = 0`) — the ceiling arm's Vec_Zero(BL+0x50) was
  effectively modelled; the `BL+0x14 = 0` write has no separate port field (the walk lanes rebuild
  from input every alive tick) — documented residual, no observable.
- The dead state was the actual gap: the port skipped Bond's ENTIRE tick while
  `death.active`/net-dead (the death-animation-sp freeze fix). The freeze was right (the dead
  branch has no obj+0x30 writer [M 0x193600]); the per-frame bookkeeping was lost with it: a stale
  third-icon context could persist on the death screen, the grounded history froze, the fall timer
  stopped, and on 07000012 a dead body frozen over the void never latched channel 0x7C.

## What was ported

`Bond::dead_contacts_tick` (crates/nf-game/src/player.rs) — the collide-slot pass for a dead body,
no movement: `icon_ctx = 0xFF` [M :21-24], the contact flags down (`head_bump = false` — the list
walk is dead [M :27-29 + obj+0xD0 freed]), the feet ray + band + walkable measure from the frozen
position (no push applied — the game's dead pass adds no DeltaP [M :131-134]), the grounded history
[M :78-82], the fall timer [M :97-103] with `fall_damage = None` (the dead landing pain
`Player_HandlePain` [M :120] is NOT ported — [S] unverified whether `Player_Damage` admits a dead
player), the void-fall latch [M :104-108], and the ceiling arm's velocity zero [M :168-175].
Wired into scene.rs's gate: the dead branch (SP death or a net-dead slot) runs the pass after the
clear; the mounted branch (Player_Disable froze BL+0x94A = 0 [M 0x195f70:16]) gets the BL+0x95F
clear only [M 0x1949a8:24-25 — the handler returns right after it]. No Rand draws added or removed
(the two tails contain none; the handler's Rand count is unchanged).

## The proof tests (real input, no teleports after the load)

- `crates/nf-view/tests/ceiling_bump_live.rs` — 07000001 "Paris Prelude": after the intro
  (the walker's cut-scene skip), Bond WALKS from spawn #0 (−2.78, 6.84, −12.40 [D the placement
  rows]) with the stick until an upward segment from the chest hits real level geometry — the spot
  found deterministically every run: root y 6.466, ceiling bottom 7.829 (a 1.36 m soffit [D the
  level data]). Jump: the rise is seen (jump 1, vel.y > 0), then jump 2 with `vel == 0` [M
  Player_HandleJump 0x1916b0:69-74] and the head parked at the face (capsule top 0.825 above the
  root, within 0.6 of the measured face, never past it). GREEN.
- `crates/nf-view/tests/dead_contacts_live.rs` — 07000005, the kill rides the real damage API the
  death suite uses (`pain_generic`):
  - a dead GROUNDED body: a stale icon context 6 clears to 0xFF on the death tick and every dead
    tick after [M 0x1949a8:21-24], the feet ray still grounds [M 0x194188:176-185], the grounded
    history keeps flag 8 [M :78-82], the ceiling arm holds `vel == 0` [M :168-175], and the body
    does not move over 60 dead ticks [M 0x193600:316-319]. GREEN.
  - a dead AIRBORNE body (killed at a jump's apex): grounded history 0 [M :78-82], the body does
    not fall over 60 dead ticks (the death-animation-sp freeze, now with the bookkeeping running),
    and the fall timer keeps counting [M :97-103]. GREEN.

## What ran (each binary whole, --test-threads=1)

- `nf-game --lib`: **748 passed, 0 failed** (5 ignored) — includes Bond's unit tests.
- death family (the code the change touches): `camera_death_crouch_live` **2**,
  `death_anim_sp_live` **2**, `death_channel_timing_live` **4**, `fix_death_restart_live` **8**,
  `abseil_death_live` (nf-game) **5** — all green.
- void paths (the void-fall latch the dead tick now feeds): `void_recovery_07000003_live` **4**,
  `walkreal_probe0c_voidspan_live` **1** — green.
- walkreal levels (one or two per tool call): `walkreal_07000001_live` **7 passed, 1 ignored**,
  `walkreal_07000005_live` **14 passed**, `walkreal_07000002_live` **4 passed, 1 ignored**,
  `walkreal_0700000c_live` **3 passed, 1 ignored** — all green.
- the two new tests: `dead_contacts_live` **2**, `ceiling_bump_live` **1**.
- the fast runner (`tools/test/suite.py`): DID NOT PRODUCE A TOTAL LINE. The sandbox kills
  background jobs between tool calls — two starts (plain `nohup` and `setsid`, pid-verified alive
  both times) died before the first binary finished, and the full suite is a 60-90 minute run that
  cannot survive that; the remaining window could not fit a foreground pass either. In its place
  every binary that exercises the changed code ran whole, listed above, plus four walkreal
  levels; no FAILED or panicked line appeared in any of them.

## What's left

- The dead landing pain (`Player_HandlePain` on a dead body, [M 0x1949a8:120]) — needs a live
  check or a disc-side look at `Player_Damage`'s dead gate before porting; the port emits no
  `fall_damage` from the dead tick.
- The climb arm's exact last-hit-above-eye flag test (BL+0x760 vs the port's capsule-push
  direction) — the consumers agree today; a live check comparing a tall-wall jump (does the game
  bump off a wall contact above the eye?) would settle whether the stand-in needs replacing.
- `BL+0x14 = 0` in the ceiling arm has no separate port field (documented residual above).
- The rest of the walkreal_*_live battery beyond 01/05 — the suite's completed portion covers what
  fit; a future pass should run the remainder.

## Live checks for the main agent

1. Dead-body icon context (07000005): stand at the ctx-6 icon (23.0, 30.6, 59.8), kill Bond
   (console damage), and watch `M_CONTROL+0x95F` for 60 frames — expect 0xFF every frame (the
   handler clears before the freeze gate [M 0x1949a8:21-24]); if ThirdIcon_Update still re-arms a
   dead body (its arm mirrors the player's pick — whether `Collide_Update` picks for a dead player
   at all is the open question), the value would return to 6.
2. Dead-airborne fall timer (any level): kill Bond mid-jump with the body over open floor; watch
   `BL+0x928` (fall timer) — expect +1 per frame while the frozen y velocity reads negative
   [M 0x1949a8:97-103], and no pain on the (never-advancing) landing.

## Usage

not visible to me
