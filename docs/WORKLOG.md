
### level-0a-office-hatch-4 (started 00:13 UTC, 2026-10-24)

Task: 0700000a — the spawn floor (the game's cel-scoped ray finds −0.0109 under the cab; the
port's flat ray reads 0.0000 → the spawn 1.0891 [L] vs the port's 1.1000), the west-hall glide
1.1147 (the doorway wall-base climb 0.1622 at x −8.304 settling via the band), and the corner
at (−17.87, 4.87) walked through at 1.127→1.120. Three ignored sisters in
walkreal_0700000a_live pin the [L] numbers; the main walk stays ignored. Plan: the spawn ray's
chain pick [M build_FindCel 0x190d88], the doorway line climb, un-ignore the sisters, the
corner, the main walk.
- (01:40 UTC) The −0.0109 surface hunt: the disc's placed collision tris hold NO face near
  −0.0109 or +0.0544 at the spawn column (the plane-y sweep over ALL instances of ALL maps,
  world-centre + plane-at-spawn-xz: only obj458's two 0.0000 faces + Player_1's proxy);
  the stored plane records of the obj458 floor tris read d = 0 (the plane = the vertices);
  the dynamic set has 0 tris in the spawn box; the bullet set reads a uniform 0.1154 (the
  0x1000 cage) — no −0.0109. hatch-3's "the SS cab placement #1 mesh" identification maps to
  plc#58 (the TouchOnce arrival plate, obj oDroneTrigger1, collision 0.0000..1.3953 — no
  −0.0109) and the door pieces' bottoms are −0.0055/−0.0336. The port-side spawn answer stays
  1.1000; the sisters stay ignored; the fix needs the live check (the game's spawn-ray hitdata:
  which cel/object the game's Collide_RayIntersect reports at (−3.93, 0.1, 10.40)). Probes
  hatch4_probe1-4 committed as the evidence trail. No src changes this round.
