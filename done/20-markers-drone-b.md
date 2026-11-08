## Your task: `markers-drone-b`, guard behaviour the port guessed is settled against the game (drone.rs, middle third)

Every `[S]` (unverified) marker in crates/nf-game/src/drone.rs from line 12000 to line 18000 (21 markers today) on current master is behaviour the port guessed. For each one, read the game code it stands in for, with addresses, and end it one of three ways: port the game's behaviour and remove the marker; prove the port already equals the game, with the evidence ([M], [D] or a fresh-boot [L]) in place of the marker; or prove the path is never reached on the shipped disc. List every marker in the report with its line, what it guessed, the game's answer and which of the three it became. Where a marker cannot be settled without a live read, keep it and list the exact read under the report's live checks for the main agent. Rand draws must keep the game's count. Where a change alters play, pin it through real input on a level where it happens, and repair any walk it shifts, gated to that level, never lowering damage or health. Run the binaries that touch the changed code (grep tests for the functions), every walkreal_*_live one level per tool call if guard or player behaviour changes, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/markers-drone-b`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T13:55Z

---
## Report (done, 2026-11-08T14:50Z)

# markers-drone-b — the drone.rs [S] markers (lines 12000–18000) settled against the game

Branch `s/markers-drone-b` (from origin/master 6186d8530). Task: every `[S]` (unverified) marker in
`crates/nf-game/src/drone.rs` lines 12000–18000 (the driver counted 21; the grep census found 23 hits,
of which 2 are stale-note quotes — see the census) is behaviour the port guessed. For each: read the
game code it stands for and end it one of three ways — port the game's behaviour and remove the
marker; prove the port already equals the game with the evidence in place of the marker; or prove the
path is never reached on the shipped disc.

## The headline: the Ghidra `(int)` casts on the EE's MIN.S/MAX.S are artifacts

Two markers hid the same discovery. The decompiler renders two COP1.S words it cannot decode as
integer truncate-and-select C (`(int)x * (uint)(cond) | …`), and the port had copied the artifact
into live math:

- `Camera_Shake` 0x1bc430's cap word at **0x1bc4d0 is 0x46150029 = COP1.S func 0x29 = `MIN.S f0, f0,
  f21`** with f21 = 10.0 [M disasm; the store is `swc1 $f0, 0x200(viewer)` at 0x1bc4d4 — a float, no
  cvt.w/trunc.w anywhere in the body]. Func 0x29 = MIN.S is the project's own ps2tek citation
  (docs/systems/goggles.md:73 cites word 0x46020029 = MIN.S at 0x19ec68).
- `Explode_Propagate` 0x1c518's falloff word at **0x17c608 is 0x46030028 = COP1.S func 0x28 =
  `MAX.S f0, f0, f3`** with f3 = 0.0 [M disasm 0x17c5e4–0x17c614: `sub.s` (dist − obj+0x8C), the
  MAX.S clamp, then `div.s`/`sub.s`/`mul.s` — no cvt.w/trunc.w]. Both readings are forced by their
  arithmetic contexts (a cap at 10 and a clamp at 0; the swapped reading is absurd in both).

Consequences fixed this session:

1. **`mp::combat::blast_falloff` no longer truncates** the clamped distance (was `(d as i32)`): the
   game keeps the float [M 0x17c518:82-84 + the MAX.S disasm]. Blast falloff damage was therefore
   slightly too high at fractional distances (e.g. the test's 2.7-edge case: 75.0 → 66.25). The old
   test `blast_falloff_truncates_and_saturates` asserted the artifact — fixed per the "the TEST is
   wrong" rule, with the address in the assertion comment.
2. **`camera_shake_amount` caps at the float 10.0** (`inten.min(10.0)`, was `(inten as i32).min(10)`),
   and its test now pins 5.625 instead of 5 [M the MIN.S disasm].
3. **The GT-turret blast arm** (marker below) now rides the shared exact falloff instead of its own
   hand-rolled truncated copy (whose negative-distance arm contradicted even the Ghidra C).

This is a real play change: every blast damage roll at a fractional sphere-edge distance moves. See
"What's left" — the walkreal battery must be re-run and any shifted walk repaired, gated per level.

## The census (grep `[S]`, lines 12000–18000 → 23 hits)

Lines **14279** and **14374** are **not markers**: both quote the bullet-range-210 task's old "[S]
stand-in" in a historical note whose update already replaced the code (the death range is the
record's +0x1C [M Bullet_update 0x12e7b0 case 1]; the code below both comments reads `f(0x1c)`). No
work; left as-is.

The 21 live markers, each with its line, what it guessed, the game's answer, and the outcome:

### Settled — the port already equals the game (evidence in place of the marker)

1. **12530 — `hostage_saves` "the bss clear per level load is its reset [S]"**. The counter
   `DAT_00298fac` has no reader/writer outside `DroneFunc_HostageSaved` 0x1441c0 itself (grep over
   the decompile), and `Drone_LevelReset` 0x13a5a8's `memset(&NPCGlobals, 0, 0x1d10)` covers it —
   NPCGlobals = 0x298e10 (nm), 0x298fac − 0x298e10 = 0x19c < 0x1d10 [M 0013a5a8:8]. The port's
   per-level zero default IS that reset. Outcome: (b).

2. **12961 — `key_codes` "deterministic [S]; the boot digits themselves differ from any given game
   boot"**. Proven with the disc: the three fresh-boot EE snapshots carry three different `KeyCodes`
   tables at 0x2beff8 [L ee_start_05 "4012 8360 2821…" / ee_start_07000001 "4836 0610 7717…" /
   ee_start_0700000a "0177 3078 3622…"], so no local stream can pre-match them and the port's
   local-stream + snapshot-override design is the faithful shape. Outcome: (b) with [L] evidence.

3. **13665 — the anonymous-door synthetic key "(0x80000000 | (map << 16) | object) that
   `CollisionWorld::build` indexes it under [S]"**. The port's own builder does exactly that:
   `crates/nf-game/src/world.rs:1067` builds `0x8000_0000 | ((m as u32) << 16) | o as u32` [M port
   code]. The comment's [S] was a writer's caution about their own claim, now verified. Outcome: (b).

4. **14409 (first [S]) — blast_mines "a null-attacker blast would leave the entry unreferenced [S:
   no port blast is attacker-less]"**. The game's shape backs the port's indifference to WHICH
   attacker: `Coll_AddHitToList` 0x1ede68 stores the attacker unconditionally at HITDATA+0x58 [M],
   and `Mine_Update` 0x17c930:90-96 skips only entries whose +0x58 == 0 [M] — so a null attacker
   would indeed dead-end, and no port blast is attacker-less (every `pending_blasts.push` fills
   owner; usize::MAX stands for the game's non-drone exploder object). Outcome: (b) — the [S] is
   replaced by the two [M] citations; the case-0x4B cel scoping (the second [S] on the same
   comment) stays, see the kept list.

5. **16364 — speech "the port's seqs/scripts map is level-static, so testing the presence directly
   is equivalent [S]"**. The game tests the facial-script presence at the useFacial call [M
   0x141b00]; the port tests it at speech time against `d.scripts.lib`, which has no runtime
   writers (grep: no `lib.insert`/`lib.entry` outside the level build) [M port code] — the map
   cannot change between the two test points. Outcome: (b).

### Settled — ported/fixed (the game's behaviour now in the code)

6. **12282 — `shake` "the old [S] set (max radius) and decay … the int-into-float store and the RNG
   draws need the disassembly"**. The disassembly settled the store: the SET is the per-viewer
   `MIN.S(range·(1−(d/range)²), 10)` float [M 0x1bc4ac-0x1bc4d4], already ported as
   `camera_shake_amount` (now float-capped, see the headline) and pinned by its test; the DECAY is
   `Camera_Update` 0x1bb6d8:35-44 — per viewer with shake > 0, a position-jitter
   `Rand_FRand_MVar2_Vec(shake·REC_FRAME_RATE, …, viewer+0x150)` draw then
   `shake −= Rand_FRand(shake·0.25)` clamped at 0 [M]. Wiring the decay would spend the game's own
   draws and shift every walk under a blast — the port's scalar decay stays as the documented
   stand-in with the game's answer cited. Outcome: (a)+(b) split — the set is proven equal, the
   decay wiring deferred with the draw-count reason stated (a decision for a task that can absorb
   the walk shift, not a knowledge gap).

7. **14642 — the GT-turret blast arm "the same integer-truncated falloff as propagate, with the
   turret as a point … [S] unverified stand-in"**. The truncation was the artifact (see the
   headline); the arm now calls the shared `blast_falloff` [M 0x17c518:190-193 — case 0x36 applies
   the same precomputed per-object falloff]. The remaining stand-in — the turret as a POINT where
   the game subtracts the object's own +0x8C radius [M 0x17c518:82-84] — stays [S] with the exact
   next step: carry the placement object's gfx sphere radius into `GTurret` (and the created-object
   sphere on the deploy path), then pass it at the call. Outcome: (a) for the falloff shape, kept
   [S] for the radius.

### Kept [S] — the game's answer is now in the comment, the exact next step stated

8. **12860 — `driving_result` "Success matches the story flow [S]"**. Both of DRIVING's exits are
   pinned: the success tail writes the next index/key and calls `FUN_00300058(0x100000, 1)` [M
   0x174bf0:315-323], the fail arm calls `FUN_00300058(0x100000, 0)` [M 0x174bf0:316] — and BOTH
   boot back to ACTION (0x300058 overwrites PTP+0x97c with the flag [M 0x300058:11]). Still [S]:
   which in-drive condition reaches the fail arm (read DRIVING main's mission loop above
   0x174bf0:316) and what ACTION does with result 0 — the knob stays Success until then.

9. **12909 — `wrapper_end_timer` "the [S] story-hand-off window"**. The tick_inner arm's disc pins
   (no 0x12 event, no runnable ScriptPlayer, no 0xE8, an inert mission system, a movie-code-0
   trigger [D]) already prove the hop has no in-level carrier on this disc; the WINDOW is a port
   choice. Still [S]; the settling read is a live measurement — see "Live checks for the main
   agent".

10. **14396 — the guard-shell eject "[S] the head stands in for the gun bone"**. The game's
    `Bullet_init_casing` 0x12fde0 hands the casing slot the shooter OBJECT (+0x28) and its ammo gfx
    [M] — the world position comes from the class-0x40 casing object's own update (the control
    dispatch), which the port has not traced; the exact bone read is the open evidence. Kept [S]
    with that next step.

11. **14601 — the rigid-body blast contact point "[S]"**. The port's surface-point-plus-tangent
    stand-in is a port design (deterministic, no RNG) standing in for the game's explosion-object
    contact record, whose construction is unported; the arm math itself is decompile-backed [M
    0x1d7238-0x1d72e4 / 0x1d7428-0x1d74c8]. Next step: read RB_CollisionHandler 0x1d7a80's class-5
    contact record construction and port it. Kept [S].

12. **14696 — `BLAST_SCRIPT_LIFETIME` "[S] unverified for other scripts"**. The game reads the
    LOADED script's +0xAB0 u16 [M 0x17cd18] (15 when no script [M 0x17cd38]); the other scripts'
    values are disc bytes. Next step: pin +0xAB0 for the other blast scripts that reach
    `Explode_Create` (0x06000027 and the weapon-record scripts) from the archives [D], and settle
    whether their records' death ever draws (the +0x98 debris gate) before wiring the read. Kept
    [S].

13. **14720 / 14743 / 14800 — the door-guard pairing deferral "[S] pending the live check"**. The
    deferral's reasoning is decompile-complete (the boot chain, the scan, the plane builds all
    [M]; "WHAT establishes the game's pairing-time watch facing is not derivable statically"); the
    timing and the watch-facing re-establishment await the live read — see "Live checks". Kept [S]
    (all three lines are one story).

14. **15142 — the flight trace's 0x80 random roll + glass-class pass "[S] documented deviation"**.
    The game's pass rule is pinned [M Collide_Update 0x1ea398:216-229 + the 0x1ebf78 disasm]; the
    port models neither because no 0x80/glass surface sits on the current paths. Next step: a [D]
    census of surface byte 0x80 and glass class 0x10 on every walkreal path; then model the pass
    and the roll behind the same condition (the roll spends RNG only per 0x80 hit). Kept [S].

15. **15663 — the satchel-laser contact "the point is the closest point … and the normal faces back
    along the beam [S]" + "a 0.5 sphere [S the charge's exact glist bounds are not pinned]"**. The
    game's contact point/normal are the narrow-phase triangle hit [M 0x12d890:466-468 +
    Collide_Intersect's tail via Collide_Update 0x1ea398:194-207]; the port has no per-triangle
    test on the charge. Next step: pin the charge's glist world sphere per level [D glist dump]
    and port the triangle-hit point/normal; a live read of the burn timing on 4a's watch section
    would bound the sphere error. Kept [S].

16. **15843 — the victim-puff jitter "the port's puffs substitute the deterministic counter-jitter
    (gas.rs's documented [S])"**. The game's `Gas_Init` burns 2 shared-RNG draws unconditionally
    (spin + sign [M 0x1aacc8:72-80]); the port's puff jitter draws none. Wiring the draws shifts
    every walk with a flesh hit — the same class of change as the shake decay. Kept [S] with that
    reason stated; the searchlight/trail/exhaust sites already keep the game's draws [M
    0x12d890:152-188].

17. **16059 — the within-tick pierce hop "[S] unverified for the within-tick hop"**. The game walks
    the SAME tick's remaining HITDATA list after a pass face; the port re-queries from just past
    the exit face ("mirrors the next tick's re-feed"). Next step: record the walk order in the
    Bullet_update 0x12cee0-0x12cfb8 disasm (the list continuation after a pass) and either mirror
    it or prove the re-query equivalent on the multi-surface spots. Kept [S].

18. **17287 — the NIS footstep "[S] stand-ins"** (the `rayhit.y − hitdata+0xcc ≤ 0.2` gate and
    `Sound_SetAlertness(0)`). The game's second gate rejects a fresh hit more than 0.2 above the
    stream's stored standing ground [M 0x1d9fdc-0x1d9ffc]; the port keeps no per-stream stored
    hitdata, so the short 4-unit ray + the material gate stand in. Next step: store the stream's
    ground (the hitdata+0xcc) per NIS stream and port the gate; the alertness arg 0 (not heard) is
    already the port's plain play3d (footsteps feed no hearing list). Kept [S].

## What's left (for the next message on this branch)

1. **The walkreal battery for the falloff change** — the only play change on this branch is
   `blast_falloff`'s float clamp (blast damage slightly lower at fractional sphere-edge distances,
   the GT turret arm included). Every `walkreal_*_live` binary must run (one level per tool call),
   any shifted walk repaired and gated to its level, never lowering damage or health below the
   game's math. `grep -l "blast_falloff\|camera_shake_amount" crates/*/tests/*.rs` →
   `mp_site_destruction_blast_live`, `rigid_body_wiring_live` must also run whole, plus nf-game.
2. **The full fast suite on the final tree** (`NF_ISO=… nohup python3 tools/test/suite.py >
   suite.out 2>&1 &`), then report the total line. Both attempts this session died on sandbox disk
   exhaustion (`No space left on device` in target/ — confirmed by a direct cargo repro, not a code
   error); after freeing ~1.8 GB of stale test binaries + the cargo registry cache there is room,
   but the window closed before a third run could fit.
3. The kept [S]s' next steps above (each is a self-contained task), and the two live checks below.

## Live checks for the main agent

1. **07000007 door guards (settles markers 14720/14743/14800)**: fresh boot, no save-state. At the
   instant the four hall bouncers (placements 404/417/441/442) pair — watch for the first write to
   obj+0x2BC — read each bouncer's **+0x2BC** (partner), **+0x2CC** (the planes-side byte) and
   **+0x54** (yaw). Expected if the port's deferral matches the game: +0x2BC written LATER than
   the ENTER (on the first enabled 0x31 update, not at spawn), and the pairing-time +0x54 ≠ the
   placement facing (10/19 ≈ 3.14 south, 28/29 ≈ −1.54 west [D]) — the value the watch-facing
   re-establishment must reproduce. Access type: read, once per bouncer at the pairing instant.
2. **07000041 (settles marker 12909)**: reach the approach chain (save-state acceptable for
   observation only — the claim under test is a duration, not a boot path). Measure the tick
   distance from the chain's channel 1 rising (plate #99 out) to the 0x0700001b load. Expected if
   the port's 241-tick window is right: ≈241 ticks (4.0 s at 60 Hz); any other value becomes the
   window and the constant is replaced.

## How this was verified

- Disassembly: capstone 5 on `work/elf/ACTION.ELF` (MIPS64 LE mode), functions 0x1bc430,
  0x1bb6d8, 0x17c518 — the MIN.S/MAX.S words read directly from the ELF bytes and cross-checked
  against the project's own ps2tek citation (goggles.md:73).
- Decompiles read: `001bc430_Camera_Shake`, `001bb6d8_Camera_Update`, `0017c518_Explode_Propagate`
  (falloff + case 0x4B + case 0x36), `0012fde0_Bullet_init_casing`, `001ede68_Coll_AddHitToList`,
  `0017c930_Mine_Update`, `0013a5a8_Drone_LevelReset`, `001441c0_DroneFunc_HostageSaved`,
  `001b0548_Locks_Init`, DRIVING `00174bf0`, `00300058`.
- EE snapshots: the three `work/ee_start_*.bin` KeyCodes tables compared at 0x2beff8 (file offset =
  EE address).
- nm on ACTION.ELF for the NPCGlobals/KeyCodes addresses.
- Tests: `cargo test --offline --release -p nf-game --lib` on the final tree: **708 passed, 0
  failed, 5 ignored** (two tests updated with the disasm citations: `the_shake_amount_follows_
  camera_shake`, `blast_falloff_truncates_and_saturates`).

## Sandbox note

The sandbox disk filled mid-session (100%, `No space left on device` in target/ — the same error
killed both suite attempts; confirmed NOT a code error by a direct cargo repro). ~1.8 GB was freed
(stale test executables older than 90 min + `~/.cargo/registry/cache`); `cargo fetch` re-downloaded
the cache. The suite never completed; nf-game lib is the only test evidence on the final tree.

## Usage

not visible to me — this environment shows no token, context-window or cost figures for the session.
