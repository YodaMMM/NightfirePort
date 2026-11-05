# level-1b-drake-cutscene-cast — Drake missing from the 1b cut-scene

Drake does not appear in the 0700001b cut-scene he belongs in. Find which cut-scene and cast row should place him, trace why the port skips it, fix it, and add a test that reads the cast after the scene starts. Check first whether s/level-1b-platforms-and-drake-scene already covers this; if it does, say so with evidence and stop.

---
claimed-by: sandbox5 at 2026-11-05T07:54Z

---
## Report (done, 2026-11-05T08:04Z)


# level-1b-drake-cutscene-cast — report

**Verdict: already covered.** `s/level-1b-platforms-and-drake-scene` — merged into master in
339a42729 (batch-1101i, 2026-11-01) — covers this task's subject, per the task's own first
instruction ("Check first whether s/level-1b-platforms-and-drake-scene already covers this;
if it does, say so with evidence and stop"). This branch therefore makes **no code change**;
it verifies the coverage on the current master tree and records the evidence.

Branch `s/level-1b-drake-cutscene-cast` (from origin/master 27c191518). WORKLOG:
`docs/WORKLOG.md`, entry `level-1b-drake-cutscene-cast`.

## The check

- 339a42729 merges `s/level-1b-platforms-and-drake-scene` into master and carries
  `crates/nf-view/tests/platforms_drake_scene_1b.rs` (+80 lines) [M `git show 339a42729 --stat`].
- That branch's report (`docs/reports/level-1b-platforms-and-drake-scene.md`, preserved at
  a2660d9c0 — report files are dropped from master after merge, c84560da4) has
  **"Part 2 — Drake missing from the cut-scene"**, which is exactly this task.

## How the merged branch covers each ask of this task

1. **Which cut-scene and cast row should place Drake.** The level's boot cut-scene is the PACK
   script **0x0600085c** (0780001b.bin, 2000 ticks, 12 streams [D 0780001b stream decode]).
   Drake is **stream 1**: `ANIM(0x0500008f, 0x060008a5)` — skin 0x0500008f, the station
   centrepiece skin (also drone #140's [D docs/levels/0700001b.md]); the cast row is
   `script Some((0x0600085c, 1))` with `skin.hashcode == 0x0500008f`. Stream 2 is the second
   actor `ANIM(0x0500007c, 0x060008a6)`. Both skins live in 0700001b.bin on skeleton 0 [D].
2. **Trace why the port skips it.** The trace found the port does **not** skip him at the
   data/sim level — pinned end-to-end by the merged test: the Scene holds both cast rows; the
   sim ticks both `Obj::Anim` streams (frame > 100 of the 2001-tick anim by tick 401); both
   poses resolve on skeleton 0 (73 bones), Drake's root at **(2.13, −0.03, 0.44)** — the hub
   centre. So if the symptom persists visually, it lives in the **draw layer** (cel/frustum
   culling of the script actors at the origin-matrix instances, the GPU palette path, or a
   pose-to-render handoff) — which needs the main agent's GPU render (no GPU in a worker
   sandbox; merge lesson 6).
3. **Fix.** Nothing to fix in the data path: the port binds, ticks and poses Drake's actor. A
   fix would only be defined once the main agent's render check finds a draw-layer divergence
   (see "Live checks" below). Redoing the trace would produce the same pins.
4. **A test that reads the cast after the scene starts.**
   `crates/nf-view/tests/platforms_drake_scene_1b.rs::the_arrival_scene_binds_drake_and_the_pose_resolves`
   loads 0700001b.bin through the real loader (`load_with_carry_difficulty`), steps **401
   ticks** (mid-scene), then reads `scene.characters` for both cast rows
   (`script == Some((0x0600085c, 1/2))`, skins 0x0500008f/0x0500007c), asserts the sim's
   `Obj::Anim` streams exist and tick (frame > 100), and asserts both poses resolve on
   skeleton 0 with 73 bones.

## Verified on this branch today (tree = master 27c191518)

```
cargo test --offline --release -j2 -p nf-view --test platforms_drake_scene_1b -- --test-threads=1
running 2 tests
test an_idle_bond_breaks_no_destroyable ... ok
test the_arrival_scene_binds_drake_and_the_pose_resolves ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
```

The repo's own level doc corroborates the scene id on master: `docs/levels/0700001b.md` calls
0x0600085c "the arrival script" (its streams 6/9 write the ch 190/191 gates) and lists
drone #140 with skin 0x0500008f.

## Related branches checked, not this task

- `s/level-1b-drake-4` (and the earlier drake-2 / drake-3 / drake-walker-kill /
  drake-round-lifetime tasks) are the Drake **round/defeat-lifetime** chain (ch 0x0e, the
  defeat NIS, the walker kill), not the cut-scene cast placement. No overlap with this task's
  subject.
- No open branch named `s/level-1b-platforms-and-drake-scene` remains on origin — it was
  merged and deleted; its content is in master.

## Not done, and why

- No code change: the task's instruction is to stop with evidence if the older branch covers
  it. This branch = origin/master 27c191518 + two docs files only (`docs/WORKLOG.md`, this
  report), so the suite state is master's own; the one relevant test binary was run above
  (2 passed / 0 failed). A full-suite re-run would only re-verify an unchanged tree and was
  skipped for that reason.
- No draw-layer fix: undefined until the main agent's render check below runs.

## What a follow-up task on this symptom should be [S]

A render-verified draw-layer task: the main agent renders the two spots below during the
cut-scene window; if Drake's actor does not draw while the second actor (0x0500007c) does,
diff the two cast rows through the draw path (cull state at the origin-matrix instances,
palette upload, pose→render handoff). The data-level facts are already pinned in the merged
report and its test.

## Live checks for the main agent

Already named by the covered branch's report (quoted here for convenience):

1. **Render (Drake)**: `--bond 12.0,-1.9,-1.0,0.2,0.0` during the arrival cut-scene window
   (the first ~2000 ticks / before the controls free): Drake (skin 0x0500008f) should stand at
   the hub centre (≈(2.1, 0.4, 0.4)) at his console. Also `--bond 2.1,1.0,6.0,3.14,0.0` looking
   at his spot. If Bond's actor (0x0500007c) draws and Drake does not, the render difference
   between the two cast rows is the bug; if NEITHER draws, the script-actor draw slot is the bug.

## Usage

not visible to me
