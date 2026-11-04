# real-health-14-00agent-4 — the 14 00 Agent walk past the mezzanine lip

Branch `s/real-health-14-00agent-4` = origin/s/real-health-14-00agent-3 (7ceaffa18) merged
with origin/master. Status at the end of this session: **the honest ref rule (the task's
lib fix) is LANDED, unit-tested and probe-proven at the lip; the coupled re-land chain was
landed and measured with it and does NOT close yet — the tree is parked at the -3 tip's
green set with the flag shipped default-off.** Everything below is pushed.

## What's done

1. **The push gate is settled and now LANDED as code.** The -3 report's verdict (the
   ref-guard in `Intersect_CylGeom` 0x109da0:431-447, not the +0x51 flag) was re-verified
   against the decompile this session [M work/decomp/ACTION.ELF/00109da0: the push, the
   pick bit (`*(ushort*)(param_1[9]+0xc) |= 1`) and the best-hit record tail
   (`LAB_0010a738`) ALL sit inside the `0.0 < dot(q − HITTEST+0x50, n)` &&
   `|q − +0x50| < radius` arm; a clean candidate that fails the guard skips the triangle
   entirely]. The port now has that rule: **`Bond::honest_ref`** (crates/nf-game/src/
   player.rs, opt-in, default `false` = master behaviour for every other binary) makes
   `capsule_tri_contact`'s failed ref-guard return **no push and no contact** instead of
   the [S] spine through-push fallback (crates/nf-game/src/world.rs, the arm the -3
   report pinned). The `capsule_push`/`capsule_push_scoped` signatures carry the flag;
   the rb/pick paths pass `false` (the bodies' ref is their own live +0x50 [M
   0x1d741c-0x1d7420] and keep master's shape). A new unit test pins both arms
   (`the_honest_ref_rule_skips_the_contact_where_the_guard_fails_and_pushes_radially_
   where_it_passes`: far ref + honest → zero push, no walkable bit; near ref → the radial
   push along (q − ref) [M 0x109da0:447]; honest off → the fallback still fires).
2. **The lip mechanism is PROVEN, with the holding contact identified.** New probe
   `crates/nf-view/examples/probe14_lip_release.rs`: Bond posed crouched at the mezzanine
   west lip (45.13, 3.05, 39.87) under 600 raw forward ticks. With `honest_ref = true`
   the lip **releases in 28 ticks**; with `false` (master's fallback) x pins at
   **45.1338** — the -3 report's exact number, reproduced. The NF_CAPS_LOG trace shows
   the hold is the **lip's west face (n = (1,0,0)) fallback radial push** cancelling the
   westward crawl each tick; the two boundary contacts (the floor and the face at
   d = r = 0.55 exactly) push zero. That is the live-push vs dead-tail divergence,
   measured at the mechanism level.
3. **The coupled re-land chain was landed and measured with the flag — it does not close
   yet.** The cc fix (bd007c18f) and the re-land (74f660327: the strafe drift, the 3.6
   vent windows, the lid crouch toggle) were cherry-picked (4f6352d7a, fbdded0c6) and the
   d2 `with_real_damage` wrap applied, with `honest_ref` set on the walks. Measured whole
   (`walkreal_07000014_live`, release, `--test-threads=1`):
   - honest on the `_at` family only: **2 passed / 7 failed / 1 ignored** — the d1
     real-damage walk **falls out of the world at tick 745** near the spawn
     ((81.18, −157.15, 62.79), 100 fall damage, "outside the hit record") — a
     NON-up-facing push loss near the spawn (the up-facing floor carve-out did not save
     it), and the four non-`_at` walkers (the vent drop, the E-switch, the skipped
     cut-scene, the tripbomb route) still jam at the lip because their bodies did not
     carry the flag;
   - honest on every walker in the file: **1 passed / 8 failed** — worse.
   The chain therefore stays **parked in history** (the cherry-picks are on the branch;
   the working tree returned to the -3 tip's byte state, verified by measurement below),
   exactly the -2/-3 park pattern, with this session's added knowledge.
4. **The parked tree is the green set, measured whole this session:**
   `walkreal_07000014_live` **9 passed / 0 failed / 1 ignored** (196.9 s — the d1
   real-damage walk GREEN, the driver's bar), `fix_npc_doors_live` **1/0**,
   `fix_guard_misc_live` **6/0**, nf-game lib **659 passed / 0 failed / 5 ignored** (the
   crate the lib fix touched, whole). The 1 ignored is the d3 00 Agent walk (its
   `#[ignore]` note stands; the un-ignore needs the chain closed).

## How it was verified

- Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), on the exact tree
  named; `git status --short` clean before each commit; only the task's files staged.
- The lip probe ran both arms in one binary (the honest flag on/off), pose held constant
  except the flag — the divergence is the flag's direct consequence.
- The far-ref unit-test call sites (9 in world.rs, 16 across 10 nf-view probe/test files)
  were updated to the new signature with `false` — they encode the fallback arm and keep
  their meaning; `cargo test -p nf-game --lib` finds no other broken literal (lesson 9).

## What's left (the next window's levers, in order)

1. **The d1's honest-fall is the chain's root blocker**: at tick 745 the walk falls out of
   the world near the spawn with the non-up-facing fallback pushes killed. Identify the
   holding contact (the probe pattern generalises: NF_CAPS_LOG on the first 800 ticks of
   the honest d1 walk, diff the PUSH lines against master's) — if the game's Bond survives
   that spot by the move-clip (the port's `capsule_tri_clip`, measured breaking earlier
   legs alone in the -2 report), the clip+flag pair is the model to land; if the spot is a
   specific rim, a scoped stand-in note goes there.
2. **Re-apply the chain on top**: the cherry-picks are one `git cherry-pick` away
   (4f6352d7a, fbdded0c6) plus the d2 wrap hunk (the 55f0b4055 diff, three lines); set
   `honest_ref` on the four non-`_at` walkers' bodies as well; re-measure the 2/7 → the
   lip legs should clear (the probe proves the release), the trench/vent legs ride the
   drift+windows as measured in the -3 report's states.
3. **Then the d2 wrap green and the d3 un-ignore** (the 1.72-landing wedge is the cc
   fix's target; the fights' 50.0-by-tick-26340 budget line needs the smoother early game).
4. The full four-crate suite did not fit this window (the time went to the mechanism
   probe and the four state measurements); the parked tree's changed-code set is covered
   by the runs above (nf-game lib whole, the 07000014 binary whole, doors + guards), and
   every other binary is byte-identical in behaviour to master by the flag's default.

## Live checks for the main agent

1. **The mezzanine west lip (07000014, x ~45.1, z ~39.9), crouched walk west** — does the
   game's Bond walk off the lip and drop the ~0.25 onto the connector without stopping?
   (The port's fallback holds x 45.1338; the honest rule releases. A game pass confirms
   the dead-tail reading at this spot and clears the chain's re-landing.)
2. **HITTEST+0x50 (BLData+0x790) through a fresh-boot 07000014 spawn-to-mezzanine play**
   (gdbwatch.py, read 4 f32s): does it stay ≈(0,0,0) on walked lines (the -3's boot
   evidence [L ee_start]) or track the player? This decides whether the game's push tail
   is dead for the spawn-area holds too (the d1 honest-fall's spot).

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere
in this environment's interfaces for this session.
