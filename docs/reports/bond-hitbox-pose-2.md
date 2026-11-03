# bond-hitbox-pose-2 — crouched hit boxes without changing the view height

Branch `s/bond-hitbox-pose-2` = `origin/s/bond-hitbox-pose` (f9bb4420) + `origin/master`
(1b2309ea) merged (82ccda59; the one conflict, bullet.rs, took master's `segment_bullet`
world trace [M Bullet_update 0x12e7b0 / Collide_Update 0x1ea398:216-229] + the -pose branch's
posed-box player walk and NF_BULLETPLOG). The task: the -pose branch's crouch toggle moved
`bond.pos[1]` down 0.4847 (ROOT_HEIGHT − CROUCH_ROOT_HEIGHT) instantly, on top of the eased
`crouch_drop` (0..0.45) that `head()` already subtracts — the camera read ~0.93 lower in one
tick instead of master's eased line. Keep the view exactly master's, keep the boxes crouched,
keep the named suites green, no mp16 re-pin churn.

## The fix (the task's first option: the offset moves out of `pos` into `bond_boxes`)

1. **The toggle no longer writes `pos`** (player.rs): the -pose branch's two
   `if self.grounded { self.pos[1] -= / += ROOT_HEIGHT − CROUCH_ROOT_HEIGHT }` blocks are
   removed; the toggle writes `crouched` + `cc` only — exactly master's `Player_HandleJump`
   behaviour [M master's toggle; the -pose diff added the pos writes in 5da2b3aa].
2. **`bond_boxes` carries the root offset** (damage.rs): when `crouch`, the palette's
   root-local frame shifts down by ROOT_HEIGHT − CROUCH_ROOT_HEIGHT = 0.4847 before posing —
   the game's settled crouched root is floor + cc + 0.051 = 0.607 vs the standing 1.086 [L
   the truck ride; the CROUCH_ROOT_HEIGHT note]. The BOND_CROUCH_POSE locals are unchanged
   (they were derived root-anchored-out assuming the crouched root), so the crouched boxes
   ride where the -pose branch put them, while `pos`, the capsule and `head()` never move.
3. **`head()` is master's line unchanged**: `pos[1] + EYE − crouch_drop` (the doc comment
   updated to say where the body pose lives).

## The proof the view is master's — a bit-exact worktree diff, then a 110-tick pin test

- A scratch probe (a flat floor; 30 settle ticks; one crouch press; a 70-tick hold — past the
  toggle's 60-tick lockout so the stand press can engage [M]; one stand press; 40 ticks)
  printed `pos[1]` / `crouch_drop` / `head()[1]` / `cc` as f32 bits per tick. It ran on a
  throwaway **origin/master 1b2309ea worktree** and on this tree: **111 lines × 2,
  BIT-IDENTICAL** (diff clean). The probe was deleted, not committed.
- The committed pin test `player::tests::crouch_view_height_matches_master_per_tick` pins
  those captured values as f32-bit literals — 70 crouch ticks + 40 stand ticks ×
  (pos, cdrop, head), every assert `==` on the bits, no epsilon. The capture also documents
  what master's own crouch does (the pin is against reality, not the naive model): the
  toggle un-grounds Bond on a flat floor (the band `hit − 0.1 <= cc` [M Player_FeetOnPoint
  0x194188:106] fails at the standing root), gravity walks the root down to the crouch rest
  ≈ 0.65 (= cc + 0.1, the band's 1-ulp edge), the view eases ±0.0225/tick to 0.45 [M], and
  the stand-up re-grounds and eases home. The first probe run (a 40-tick hold) caught a real
  subtlety: inside the lockout the stand press is refused [M the toggle's lockout] — the
  test's hold runs past it, as a player's does.

## The named suites (all run, all green, on the final tree)

| binary | result |
|---|---|
| `fix_bond_damage_live` | **7 passed / 0 failed** (5.7 s) — incl. the crouch round test: the standing-head line still flies above the crouched head box (the boxes ride the palette offset) and misses |
| `anim_events_live` | **4 passed / 1 ignored** (0.4 s) |
| `bond_boxes_every_level_live` | **1 passed / 0 failed** (20.1 s) |
| `mp_perf_16_live` | **2 passed / 0 failed** (10.3 s) at `PINNED_DIGEST = 0x89cb_fcb6_ae45_c2ac` — the master-lineage value |
| `nf_game --lib crouch_view_height_matches_master_per_tick` | **1 passed** — the 110-tick bit pin |

**The mp16 pin**: the -pose branch re-pinned to 0xb270_68a8_2582_fec6 because its root drop
moved the scripted humans' sim positions (its commit abb44f33 bisected the digest move to the
crouch-pose commit 5da2b3aa). This task REVERTS that sim change (the root drop), so the
digest returns to the pre-crouch value 0x89cbfcb6ae45c2ac — the same value master reproduces
[the capsule-cull-11 report's master crosscheck]. This is a revert of the -pose branch's own
re-pin, not a new pin from view churn: after the fix the sim's positions ARE master's, which
is the whole point. The pin comment in the test carries this history.

## The full suite

Did **not** fit this window: the sandbox spent its first half-hour on setup (the repo was
fresh — clone, work-slim, ISO, rustup, the ALSA user prefix with a fixed alsa.pc prefix and
the runtime lib), then the merge + cold rebuild (~25 min), then the two-worktree probe
capture (the only way to pin master's per-tick values honestly). **The named binaries this
task touches all ran green on the final tree** (the table above). The next message resumes
with the chunked fast runner on a FRESH state file (the previous windows' state files are
stale for this tree):
`python3 tools/test/chunk_suite.py /home/z/suite_pose2_state.json --budget 470` — background
runners die between tool calls in this sandbox family (the 9th measured), so the chunked
resume runner is the only way. Known environmental failures to expect: `fix_0c_pylon_live`
(2, no GPU), `mp_stress_explosives_live` (OOM exit −9), the 3 `disc_golden` (slim ISO).

## Merge notes for the driver

- The two `rope_guard` Player initializers (master's new tests) gained `crouched: false` —
  the -pose branch's field arriving in master's test files.
- The -pose branch's mp16 pin comment block was rewritten in place (it described the root
  drop this task reverts); the pin HISTORY comment now records both values and why.

## Live checks for the main agent

Optional, low value: a fresh-boot PCSX2 watch of Bond's `obj+0x30` through a crouch on flat
floor — the port (and master) un-ground and fall to the crouch rest ≈ 0.65 over ~18 ticks;
the game's [L] truck datum was 0.607. If the game does NOT fall (it snaps), the port's
band-edge rest rule is the place to look — but that is master's shared behaviour, not this
branch's delta.

## Left

- The full suite's chunked run (the command above); every red accounted then.
- Nothing else for this task.

## Usage

not visible to me — this session exposes no token, credit, cost or context-window figures in
any environment output I can read.
