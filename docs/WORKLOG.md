## level-0a-ignored-checks (2026-10-31, s/level-0a-ignored-checks)

Branch from origin/master (81da6c47, batch-1031l). Task: `walkreal_0700000a_live` has two
#[ignore]d tests (a doorway's walking height and a corner crossing). Read their ignore notes and
the merged capsule work's notes. Find what the game does at each spot [D collision; M the capsule
contact rules — head-on cull on the raw denom 0x10a1f0-0x10a2e8, Dist2Tri flags 0/99 0x10a330 —
and Player_FeetOnPoint 0x194188], fix the port gap with evidence or re-derive the expectation
where it was port-derived, and un-ignore what can be un-ignored; name the live check for anything
left. Run walkreal_0700000a_live and interactables_0700000a_live. Run the full suite with
tools/test/suite.py and quote every total.

## Progress (2026-10-31, ~21:08 UTC)

- Branch created from origin/master (81da6c47). Sandbox persisted from an earlier session: ISO,
  work/, queue worktree, cargo target dir all present. Suite start next.

## Progress (2026-10-31, ~21:35 UTC)

- The two ignored tests read + both researched via parallel subagents (decompile + port side).
  Decompile findings [M]: the walk path has NO clip / wall-stop / step-up — Player_Update
  0x193600:284-295 integrates the walk move unclipped; the cyl push tail is dead on 0a fresh
  boot (ref = HITTEST+0x50, the lifetime DeltaP accumulator [M 0x1ebaa8:353-373], boots (0,0,0)
  [L ee_start_0700000a 0x1b78ee0 = 00×16], walked contacts |closest| ≈ 10-25 ≫ r 0.55); the
  head-on cull is inert on the raw walk spine denom (0.7597 < 0.999 [M 0x109da0:321-322]);
  Dist2Tri flags 0/99 = face-interior / behind-plane [M 0x109770:42-64]; the only vertical
  mechanism is the FeetOnPoint band [M 0x194188:106].
- PORT GAP FOUND AND FIXED: NF_WALK_CLIP's move_v never spanned the walk — pre_move_pos was
  captured after the walk add (player.rs), so move_v = (0, gravity, 0) and the clip never
  engaged; the 2026-10-30 "corner crosses at +76 ticks" reading was wall-response-less ghosting.
  Fixed: pre_move_pos hoisted before every position move (contacts read the post-move root [M
  0x193600:284-295]). Default path inert (move_v only read under the flag).
- Re-measured with the flag LIVE: it WEDGES the west-hall through-line at the doorway (STUCK
  (−8.28, 1.03, 15.83), y decaying) and the 0a battery reads 1/6/2 under the flag — the clip
  model does not reproduce the game's doorway slide; the flip stays unlanded. Both ignore notes
  re-pinned with the addresses + two named live checks (BL+0x790 doorway dump; the walk-anim
  root → AnimObject+0xcc ride, the +0.0256 candidate). Neither test un-ignorable without it.
- Named binaries (default path, post-fix): interactables_0700000a_live 7 passed / 0 failed /
  0 ignored; walkreal_0700000a_live 7 passed / 0 failed / 2 ignored.
