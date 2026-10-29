# level-11-every-objective-2 — the report

Task: land the 07000011 mission-row walk. `origin/s/level-11-every-objective` was green but
could not merge: it asserted `!ch 0x48` as game truth resting on "no writer exists" — an
assertion may not rest on a missing writer (the port may simply lack it). Merge origin/master,
turn that assert into a `// [S] unverified` log out of pass/fail, keep every other assert, and
hunt the 0x48 writer.

## What's done

1. **The branch**: `s/level-11-every-objective-2` opened from `origin/s/level-11-every-objective`
   (bf7bdf07, the green walk) with `origin/master` merged (4f972872).
2. **The assert conversion** (`crates/nf-view/tests/walkreal_07000011_live.rs`): the two
   `assert!(!sim.drones.channels.on(0x48), …)` pins — at the stairwell bottom (old line 154)
   and at the exit (old line 174) — are now `eprintln!` logs marked `// [S] unverified — NOT a
   pass/fail assert`, with the header rewritten to say the writer absence is unverified and the
   walk pins the [M]-backed monitor states instead. **Every other assert is untouched**: the
   descent geometry, ch 0x7C's negative, ch 100's negative, `states[15] == [3, 3]` (announced)
   at both pins [M 0x1827d8 case 3], the Load Point #158 E-load into 07000012 [M 0x1b2318],
   and the 07000012 leg's positive [D] pins (#679's tile fires ch 0x48, the record completes,
   'Escape from the tower' stays announced).
3. **The 0x48 writer hunt — no writer exists on 07000011; independently verified [M+D].**
   The class census [M]: a decomp-wide grep over ACTION.ELF finds every `switch_channels`
   writer (103 files) — all parameter-driven object handlers (triggers `Trigger_Activate`
   0x1b2318 + the AND/OR muxes, `SS_Activate` 0x1a9ea8 / `SS_Operate`, `Switch_Activate`
   0x1b02a8, `Door_Create` 0x136100's starts-open arm, `DroneFunc_SetDeathChannel` 0x13e660,
   `DroneSpawner_Control` 0x137e68, `Destroy_Update` 0x12c4f8, `Pickup_Handler` 0x18e650,
   `Script_EventHandler` 0x1d94c0 events 0xD/0x11, `Mission_MonitorObjectives` 0x1827d8) —
   **no hardcoded `SC[0x48]` write exists anywhere**, and `Mission_MonitorObjectives`
   itself writes only ch 99 (fail) and ch 100 (the mission-success AND) [M 0x1827d8:70-121] —
   it READS 0x48 (case 3's completion test) and never writes it.
   The disc sweep [D, probe `walkreal_probe11_48_live`, committed]:
   - the script streams: 07000011's whole script lib decodes cleanly; the ONLY
     `Script_EventHandler` 0xD event on the level is `script 0600098a stream 6: params [98, 1]`
     (writes ch 98, not 0x48); zero 0x11 events; **zero 0x48-writing stream events**;
   - the raw placement sweep: **no placement param and no extra word equals 0x48** across the
     whole main map (this covers every trigger out/list, SS channel, switch channel, door
     ch_open/ch_lock incl. the starts-open arm, spawner channel, breakable channel, pickup
     channel);
   - the sim's parsed objects: no trigger out/list 0x48, no SS channel 0x48, no door
     ch_open/ch_lock 0x48; the drones' death channels hold one value, 0x08 — not 0x48.
   The mission's only 0x48 writer remains TouchOnce **#679 on 07000012** [D
   docs/levels/07000012.md:13, the spawn tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62)], which
   the walk asserts positively on the 07000012 leg. **Conclusion: the objective completes one
   level later by the game's own design; nothing to port** — the prior session's enumeration
   stands, now verified class-by-class rather than assumed.
4. **One new port gap documented en route** (not fixed — no observable on this level): the
   port parses `Cmd::Event` (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB
   (weapon); `Script_EventHandler` 0x1d94c0's channel-write events (0xD set-channel,
   0x11 conditional set) are not acted on. On 07000011 the only such event writes ch 98 —
   inert here — but any level whose cut-scene stream raises an objective channel would need it.

## How it was verified

- `cargo test --offline --release -p nf-view --test walkreal_07000011_live -- --test-threads=1`:
  **1 passed; 0 failed** with the logs in place (the walk itself: 3262 ticks, 0 stuck, 0 falls).
- The probe binary `walkreal_probe11_48_live` (committed) reproduces the whole [D] sweep above
  from the ISO in one run (`--nocapture`).
- The full four-crate suite (the fast runner): see the Suite section.

## What's left

- Nothing blocking the merge from this task's scope: the unmergeable assert is gone, the hunt
  is documented, the walk is green. The `Script_EventHandler` 0xD/0x11 port is a general
  follow-up (inert on this level).
- The sandbox disk filled twice during the suite build (the same ENOSPC the level-02 report
  documented); stale build artifacts were purged between attempts — no repo file touched.

## Live checks for the main agent

1. 07000011 fresh boot: stand at the stairwell bottom (≈ 14.8, 15.0, −2.6) and at the Load
   Point — confirm the HUD never announces "Objective complete" for "Gain access to lower
   office floor" before the 07000012 load (the port's [S] log says ch 0x48 never fires here;
   a live confirmation would upgrade the writer-absence from [S] to [L]).
2. 07000011 fresh boot: kill the guard whose death channel is 0x08 — confirm no objective row
   reacts (the port's parsed death channels; cheap sanity on the param mapping).

## Suite

The fast runner on the final tree (this branch = origin/s/level-11-every-objective + origin/master):
build ok (**888 test binaries in 1269 s**; the sandbox disk filled twice during the build — the
same ENOSPC the level-02 report documented; stale build artifacts were purged between attempts,
no repo file touched). The run was still in progress at the 03:14 UTC stop line. Its problem
lines, all environment, none attributable to this branch's test-only changes:

- `probe_pylon_render::probe_render_pylon_only` FAILED — wgpu `no_adapter: Backends(VULKAN | GL)`:
  the sandbox has no GPU (rule 6); the probe cannot request an adapter.
- `fix_0c_pylon_live` ×2 FAILED (the same 0c-pylon render assertions — GPU).
- `mp_stress_explosives_live` and `mp_online_failures` CRASH (exit -9) — the runner's wall-clock
  kill of the two long MP stress binaries on this slow sandbox (600 s cap, docs/tools/test-suite.md).

No `disc_golden` failure had appeared by the stop line. The walkreal_07000011 binary itself:
**1 passed / 0 failed** (quoted above). The next session should re-run the fast runner on the
final tree for the total line once a GPU box or a longer window is available.
