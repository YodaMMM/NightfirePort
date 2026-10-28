# WORKLOG

## level-0d-c5-door-laser (2026-10-28, branch s/level-0d-c5-door-laser)

Task: 07000d (Chain Reaction Part 2) C5 doors — class 0x21 Destroy objects
(placements #93 bigdoor01 / #96 bigdoor02, p0 ch 0x1f, p1 trigger 0x1f, p2 type
5/6, p3 health 1000/10000, p4 mask 0x10, p5 blast 10, p6 delay 0 [D]) — must
take beam damage over ~200 contact frames, not smash on first contact.

Plan:
- Port the 0x20 gate in Explode_Propagate case 0x21 [M 0x17c518], the
  Collide_FilterBullets 0x1ece30 ext+0x14 mask filter (player guns 0x08 blocked,
  mounted guns 0x10 pass), GunImp_Update 0x1320d8 -> Destroy_CollisionHandler
  beam damage (~50 hp/frame) + hit-colour (obj+0x104/0x105 -> 0, recover ~3/f),
  Destroy_Update colour/trigger read, Destroy_Smash 0x12bb60 (ext+0x17 latch,
  class 0x21->0x01, destroyed glist, light removal, 0x6000052 x3, channel
  write) and the same-frame channel smash of bigdoor01 [M].
- Correct the merged level-0d-gun-door-objective test that pins the
  one-contact smash.
- Tests on real input from the mount; full suite before push.

### level-0d-c5-door-laser outcome (2026-10-28)

- Ported: the Explode_Propagate case-0x21 0x20 gate [M 0x17c518:170] (Targets.destroys carries
  tag+0x14 now), the Collide_FilterBullets weapon-word mask in the destroy bullet path
  [M 0x1ece30:26-33; D words 0x0001/0x0008/0x0020 dropped, 0x0010 mounted kept], the hit-colour
  pair obj+0x104/0x105 (−5 on a 0x67 contact tick, +1 recover, death freeze [M 0x12bd48:61-95]),
  the destroy leaves as solid bullet stops (one 50-damage contact per round [L ~50/frame]), the
  merged gun-door test corrected (its one-contact smash contradicted [L] + [M]).
- asm/decomp findings: Destroy_Smash's SetGList arg = ext+0x00 = the LOD pointer or the
  create-time fallback pCelList+idx·0x50+0x50 [M 0x12bbb8 asm, 0x12c318:23-26, 0x1d1b68] — the
  brief's glist+0x50 destroyed entry; the C5 leaves have no block-0x27 LOD [D probe].
- Left: the sustained-fire harness stalls after ~5 landed contacts (heat/trigger quirk — beam
  up, no contacts); the drain test is #[ignore]d with the reason. The draw-side destroyed-glist
  hide needs a per-frame static-instance hook (the 0x3b gap's shape). p6>0 deferred channel
  writes unported (the C5 doors are p6=0).

## level-0d-c5-door-laser-2 (2026-10-28, branch s/level-0d-c5-door-laser-2)

Task: continue s/level-0d-c5-door-laser (merged origin/master 480aa3db). Two
items left: (1) the sustained-fire defect — trigger held with no look input,
~5 contacts land then the health freezes while the beam quad stays up and the
heat climbs; the game drains steadily under a plain held trigger [L PCSX2
2026-10-27: 10000 -> 7100 -> 4050 over two 60-frame bursts, stick at rest].
Find the port's fault in the no-input fire path (aim easing, the round's spawn
origin/direction once the aim settles, the fire arm's re-trigger) against
GunImp_Update 0x1320d8 [M], fix it, and un-ignore
the_beam_drains_the_c5_door_at_50_a_tick_and_the_smash_fires_at_zero.
(2) The destroyed-glist swap's draw: expose the dead/destroyed state so the
static instance stops drawing (C5 leaves have no LOD variant [D lod_word -1],
doorway ends open [L]); list the draw hook in the report for the main agent's
render.

Plan:
- Reproduce the stall in the no-input fire harness, trace where contacts stop
  landing (aim easing settle, round spawn origin/direction, re-trigger), fix
  against [M] evidence, un-ignore the drain test.
- Add the per-frame static-instance visibility hook for dead Destroy objects
  (the 0x3b channel-show gap's shape), document the draw hook for the render.
- Full suite on the final merged tree before push (fast runner).

### level-0d-c5-door-laser-2 outcome (2026-10-28)

- The sustained-fire stall root-caused: the drain test's aim loop converged on the SMOOTHED aim
  (0.02 window) while the game's ease [M 0x1320d8:136-161] leaves the raw ~0.1 rad past during a
  saturating approach — the zero-input ticks then eased the barrel past the door (the settled beam
  dug into the deck ~4 units short: beam far y 3.985 vs the door's y-lo 3.9875, probe trace). The
  port's fire path is decompile-faithful (the ease, the smoothed-matrix spawn [M :325 + :196-211],
  the every-tick re-trigger [M :107-112/:324]) — the TEST was wrong per the rules: it now converges
  on the RAW stick rest [M :124-135] and coasts the ease before the trigger (the [L] rest-drain
  shape). The drain test is un-ignored GREEN: six −1500 bursts, the heat arithmetic [M] saturates
  the hot byte at shot ~201 with the door at exactly 0, the ~200-tick recovery pause rides out, the
  kill shot takes the death catch to −50, bigdoor01 same-tick at 1000, ch 0x1f, objective 6 → 4.
- The destroyed-glist swap's draw side: the drawn 0xe1 leaves' centres recorded at build
  (Scene::destroy_centres, the group cull sphere's exact transform) and a per-frame dead gate in
  the renderer's draw filter (Renderer::destroy_dead fed from the sim in lib.rs) — the C5 leaves
  swap to an undrawable slot [D lod_word −1], the doorway ends open [L]. Render spots listed in
  the report.
- The merged-tree build fix: master's cascade test constructors gained the Destroy fields the
  predecessor branch added (mask/colour/hit67, p4 0 → mask 0). Four-crate BUILD green.
- Left: the FULL suite total line on the final tree (this sandbox kills background runners and the
  foreground run's build step consumed the window — the next message runs suite.py first thing);
  p6 > 0 deferred channel writes (carried, needs the live check).
