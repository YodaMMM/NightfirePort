# level-02-audit — the 07000002 behaviour audit

Branch `s/level-02-audit` (off origin/master 24084ef2). Sandbox 21:04–23:03 UTC 2026-10-25.

## What's done

1. **The full type census of 07000002's main map** (502 placements, `nf-extract placements-all` [D])
   against `parsemap_create_dynamic_objects` 0x1d04f0 [M], one row per type with the creator, the
   placement names, the game's behaviour and the port's module — written to
   `docs/levels/07000002-audit.md`. It extends the 2026-10-17 interactables census
   (`07000002.md`) with the kinds that census skipped: characters 0x0F (25 guards: skins,
   enable channels p3, death channels p12 — 4 protected ch 0x60 [D]), AI volumes 0xF5 (21
   `AI_ShadowVolume`), trees 0xF3, emitter 0xF2, ladders 0x22, wire 0x3E, third icon 0xFA,
   ThirdCam 0xE7, DynamicObject 0x3B, ripples 0x21, rainboxes 0x32, hints 0x33, sound/music
   triggers, the static path (0x19/0x47/0x23/0x17 — including the 0xC023 water cels the swim
   system reads), and the zero-placement kinds cross-checked against the full dispatcher.
2. **The two script players decoded** (`nf-extract nis`): SP #400's 0x06000084 (9 streams, camera
   + Mayhew/Bond anims, Events 4/5/6/7, stream 7's ch-0x47 hand-off, stream 8's keyed gate
   entity) and SP #399's 0x0600007f (7 streams, sounds 418/508, the ch-0x35 trigger). Every op
   kind both scripts use is in the port's `Script_EventHandler`/interp dispatch [M
   nis.rs:880-975] — no gap.
3. **The mission record** cross-checked (`mission::c07000002::DEF` [L ee_start_05.bin @
   0x2a3ed0]): watch channels 0x48/0x4B/0x4D, enables 0x47/0x47/0x4C, no per-objective fail; the
   disc writer for 0x48 is Destroy #14, for 0x47 the intro's stream 7, 0x4B = AND #303, 0x4C =
   AND #480; 0x49/0x4A/0x4D have no disc writer (mission/dialogue layer, shared with 01). Fail
   path = 4 protected guards → ch 0x60 [D] → `Mission_MonitorObjectives` 0x1827d8 [M].
4. **GAP-2 found and fixed**: `07000002.md` credited BOTH cable lights #390/#392 with the ch-0x82
   write; the disc gives #390 out 0 — only #392 feeds the arc chain [D param 0]. Doc corrected;
   the new `crates/nf-view/tests/destroy_channels_02_live.rs` pins all six 0xE1 out channels
   through the real level path ([M Destroy_Create 0x12c318:35]) — **passes**.
5. **GAP-1 documented (large, pre-dispositioned)**: 21 kind-0xD8 rigid-body props
   (`bucketswing`, 19 `rb_greenlamp`, `rb_redlamp`) — `rigid_body.rs` holds the init math
   [M 0x1d7ad8/0x1d7ba0/0x1d7da0] but nothing integrates it; the props are statics. Same open
   item 01 shipped with; its own task (shared across levels). GAP-3 is a note (no full
   wire-crossing drive test on 02; the mechanics are covered on their own levels).

## What ran

- `cargo build` of all four crates + every test binary — clean.
- The task's tests through the real ISO path: **destroy_channels_02_live 1 passed**;
  **walkreal_07000002_live 1 passed** (43.5 s, the full walk); **walk_07000002_live 2 passed**;
  hurt_02_live, bathhouse_*, fix_ss_grate ×4, doors/pickups/sensor suites — all green in the
  chunk run below.
- **Full suite, chunked** (`tools/test/suite.py` semantics, run in foreground chunks because this
  sandbox kills background processes between tool calls — three nohup attempts died): **329 of
  the 806 test binaries ran, --test-threads=1, NF_SILENT=1**: ≥497 tests passed, 62 ignored,
  0 failures beyond the environmental two binaries — `disc_golden` ×3 (allowed on the slim ISO)
  and `identity_disc` (pins the retail disc's 1,191,072 sectors; this image is the slim on-foot
  cut at 518,435 [D measured] — same environmental class; my diff touches no source). The two
  slow binaries (`weapon_matrix_live`, `mp_stress_explosives_live`) re-ran green at the suite's
  600 s per-binary limit. **477 binaries did not get a slot** — the sandbox kills background
  jobs and each foreground call caps at ~9.5 min; none of them touch this task's diff (docs +
  one new independent test file), and every binary that loads 07000002 ran green. The next
  session can resume the remaining binaries with `tools/test/suite.py` directly.

## What's left

- Run the remaining 477 test binaries (no failures expected; the merged tree = master + docs +
  the new test).
- GAP-1 (rigid bodies) as its own task; optional GAP-3 wire-crossing drive test on 02.

## Files

- `docs/levels/07000002-audit.md` (the audit, GAPs 1-3, the fix section)
- `docs/levels/07000002.md` (the GAP-2 correction)
- `crates/nf-view/tests/destroy_channels_02_live.rs` (new, green)

## Usage

not visible to me — this session's driver interface shows no token, context-window or cost
figures.

## ntfy failed (partially — delivered on a minimal retry)

At ~22:55 UTC the three fallbacks errored in order:

1. `curl` (long payload, -m 20) → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib` (ASCII payload, timeout=20) →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget` (same long payload) → no output, killed at the 60 s tool timeout.

A retry at ~22:58 with the minimal body `-d "level-02-audit: done"` **succeeded** — ntfy
accepted it (`{"id":"HrYU4KEYe5Rl","topic":"nightfire-yoda-8k3x","title":"NightfirePort"}`),
so the notification requirement is met; the long one-line result text is carried by this
report instead.

The queue finish step itself succeeded (`finish.sh` printed `OK done 20-level-02-audit`; the
`cat: docs/reports/level-02-audit.md: No such file or directory` line above it is the script
resolving the report path from the worktree root — the file is on the branch at
`docs/reports/level-02-audit.md`, commit 87793363).
