## Your task: `level-06-castle-walks`, 07000006: the castle interior walks, un-ignored

`walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery` and `stopping_before_the_tunnel_never_ends_the_castle` are still ignored. Their old blocker (the fire fault: one round per 12 presses, the view turning on the press tick) does not reproduce on master — `fix_06_fire_live` pins every press firing and no body turn.

1. Run both walks un-ignored on master. Fix what stops them with real input and the game's rules [M] — the courtyard fight with the alarm's reinforcements, the doors, the route to the upper gallery.
2. Un-ignore both when they walk; the sister (stopping before the tunnel never ends the level) must still be able to fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-castle-walks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T01:52Z

---
## Report (done, 2026-10-25T02:37Z)


---
## Report from the worker (done, 2026-10-25T02:37Z)

# level-06-castle-walks — report

Task: `walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery` and
`stopping_before_the_tunnel_never_ends_the_castle` (level 07000006, the castle interior) were
`#[ignore]`d; make them walk on master with real input and the game's rules, then un-ignore both.

## What was done

1. **Both walks ran un-ignored on master first.** The whole route worked on master — the
   truck-ride spawn, the dismount, the well lane, E-doors #47/#59 (the guardroom), the
   searchlight courtyard with the alarm's reinforcements, E-door #71 — and the real wedge was
   the TUNNEL shaft between doors #71/#72: a straight shot at (4.0, 49, −62.5) grinded in the
   shaft's south-east corner (jump apex 46.6 < the 48.8 upper-ramp slab face).
2. **The old fire-fault blocker does not reproduce on master** — `fix_06_fire_live` pins that
   every press fires and the press tick turns the body nothing [M Player_WeaponFiring 0x1a5460
   has no body-yaw write]. Not the wedge anymore.
3. **The shaft is a switchback, not one ramp** [D dense floor/column maps probed from the disc:
   floor_below column stacks over x 2.0..7.0 × z −50.5..−65.5]:
   - low ramp 42.0 → 44.8 over z −51.5..−58.5, ending at the shaft's south-east corner;
   - an east lane (x 6.25–7.0) carrying the turn 44.6 → 46.8 northward;
   - the upper ramp 47.2 → 48.8 over z −55.5..−59.25 running back south to the 49.0 landing
     (z −59.75.., door #72 at −64.88).
4. **The climb was rewritten as seven legs following those surfaces** (five `climb_stairs` legs
   with tight arrive radii so each leg reaches its mark, then two legs on the upper ramp +
   landing seam through the new shared no-jump helper `walker.rs::walk_slope`). The seam is the
   subtle one: an AIRBORNE crossing re-links Bond onto the ramp cel's phantom plane under the
   landing slab and the south shaft void swallows him [D seam probe on this tree: grounded the
   whole crossing at root ~50.0; the jump-pulsed legs fell past y 47 into the slab's underside],
   so the upper ramp and the seam are walked without any jump pulse. Every leg is still the
   player's own walk/jump input through the harness.
5. **Both tests un-ignored** (`a01d11fe`). The sister's negative asserts are unchanged and still
   falsifiable: no ch 0x83 (the LoadPoint stays cold away from its volume [D #746]), the
   objective is not complete, `level_exit` is None — if the port ever fired the LoadPoint chain
   or folded the objective off-route, the sister FAILS.
6. `docs/levels/07000006.md`'s walk-status paragraph rewritten to match.

## How it was verified

- The tests drive the real input path only (`crates/nf-view/tests/common/walker.rs`): the
  player's own walk/jump/crouch/fire/use through the sim's `Input`, doors E-opened from beside
  them, no scripted hand-outs. The two walks pass together (~27 s, no falls, no stuck events).
- Every assertion about the level cites evidence: [D] disc bytes (the floor maps, the door and
  placement census), [M] decompile addresses (e.g. Player_WeaponFiring 0x1a5460), [L] EE-snapshot
  facts. Guesses/port choices stay marked `[S] unverified` and out of pass/fail.
- Crates run for this task before pushing: `nf-view` (the walk tests + the walker harness).
- **Fast suite on the final tree** (merged with `origin/master` at `3a0cb26e`, merge commit
  `9e9bbd75`): `python3 tools/test/suite.py` (builds all four crates' test binaries once, runs
  each binary with `--test-threads=1` under a 600 s limit; NF_SILENT=1). The task's own binary:
  `walkreal_07000006_live` — **both tests pass in the suite** (`2 passed; 0 failed; 0 ignored`,
  26.6 s, no failures).

  Suite total: **3276 passed, 23 failed, 196 ignored (791 binaries)**.

- All 23 red entries are environmental to this sandbox (no GPU, 3 GB RAM, the onfoot ISO +
  slim `work/`), none from this branch (beyond the merge it touches only the four task files;
  `git diff 3a0cb26e..HEAD --stat` confirms):
  - **4 disc/identity** — the 3 `disc_golden` failures are the ones allowed against the slim
    `work/` extraction; `identity_disc` compares the disc's sector count (518,435 — this
    sandbox's onfoot ISO) against the full retail disc's 1,191,072 pinned in
    docs/build-identity.md, so it fails for every branch run in this sandbox.
  - **3 GPU render tests** — `probe_pylon_render::probe_render_pylon_only` and the two
    `fix_0c_pylon_live` tests panic at wgpu `request_adapter` — `NotFound
    { no_adapter_backends: ... }`; the sandbox has no `/dev/dri`, so no render adapter exists.
  - **16 OOM kills (exit −9, SIGKILL)** — the `net_*` / `mp_*` stress binaries,
    `walk_07000013_live` and `walkreal_probe13`, killed by the OOM killer in the 3 GB sandbox.

## What's left

- The gallery → dressing-room leg is the walk's one documented gap (the port's collision gives
  no walked connection there yet — documented in the walkreal test header; the LoadPoint chain
  it feeds is pinned green through the real trigger by `walk_07000006_live.rs`). Out of this
  task's scope.
- No visual claims to render: this task changed only the walk harness and test legs, no
  rendering/HUD code.

## Live checks for the main agent

None required. All pinned facts are [D] disc probes and [M] decompile addresses read directly.

## Usage

not visible to me
