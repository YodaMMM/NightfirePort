# real-health-00agent-06-2 — the report

Task: finish the 06 00 Agent sister (`walk_the_castle_interior_to_the_party_real_damage_00agent`).
Continued from `origin/s/real-health-00agent-06` (c356ff22), origin/master merged (b56b9781 — the
door-route guards, the 03/11 walk ignores, the real-health batch merges). Real input only; no
top-ups; the helpers are level-local (no shared-helper changes).

## Status: NOT finished — the 00 sister now reaches the courtyard descent and dies in the deck-guard lane

The prior session's walk died at the courtyard mouth with the fusebox cut still to run. This
session re-measured on the merged tree, changed the route where the merge changed the game's
facts, and staged the aimed-stand tier the prior report named. The 06 binary is 6/7 (only the 00
sister red); everything else on the touched file is green.

## What's done (all committed and pushed)

1. **Branch + merge + re-measure (the brief's "re-measure first").** `s/real-health-00agent-06-2`
   cut from `origin/s/real-health-00agent-06`, origin/master merged clean. The merged-tree run
   (run 1) changed the picture in two ways:
   - The reinforcement pair (#19/#20-class) now spawns and fires DURING the guardroom/cut phase
     with ch 57 ON (the ledger: #20 skin 0x500002f, frames 10563-10639) — the merged master's
     door-route guards re-trip the courtyard's ch-55 volume during the fights, so the cut's core
     value at 00 (silencing #735 so the pair never spawns) no longer holds on this tree.
   - The cut's exposed window itself cost 51.0 hp in one fight (#28 at the nook: 89 → 38, the
     return-fire stand).
2. **The fusebox Laser cut retired at 00** (`walk_to_the_courtyard_at(&mut w, false, difficulty)`
   — was `difficulty >= 3`): with the pair spawning anyway and the alarm not a fail channel
   [L record 1; M 0x1827d8:98-107], the cut is pure cost. Effect measured: Bond now reaches the
   balcony at 89 hp (was 38) and the mouth alive. The difficulty-1/2 walks are untouched (they
   already passed `cut = false`), and the heads assert now expects the lit heads at every
   difficulty (the cut arm deleted with the measured reason at the site).
3. **The mouth aimed-stand tier built and staged** (level-local in the 06 test file — no shared
   helper changes, the merge lesson 7): `aimed_stand` (the 0c yard stand's mechanic with its
   measured parts cited — the aim-raise loop [M 0x1a24c8:113-152], the aim-bit tap gate
   [M Bullet_init 0x12c5a0:139-170], the cursor-aware convergence [M Player_AutoAim
   0x1a7c98:50-59], the still gate 18 ticks, the strict gun-ray exposure re-pick) +
   `aimed_stand_the_mouth` (any-height alive-hostile pick within 40 u with exposed rays, the
   recent hitter first, cap 2400, the 1500-tick no-hit wait for the far deck posts). Staged at
   four spots measured from the death ledgers: the balcony's west reach (10.5, 47.1, −21.8 —
   run 1 died at the drop's edge (8.54, 47.13, −21.81) before the old mouth tier could run),
   the plaza after the mouth plate, the strip split (6.0, 42.0, −34.0 — the pair's posts
   (11.6, 42.0, −35.1)/(13.2, 41.7, −34.8) [D census]), and after the descent nav.
   MEASURED: the stands kill (the pick censuses show #28/#10/#18 alive-with-rays then gone) —
   the mechanic works; the two rays1-exposed candidates at the mouth (#28 d 15.3, #31 d 27.2)
   and the strip (#31 d 16.0) were taken.
4. **The whole `walkreal_07000006_live` binary on the final tree: 6 passed / 1 failed** — the
   failure IS the 00 sister; `walk_the_castle_interior_to_the_party` (Operative),
   `..._real_damage` (Agent), `..._real_damage_agent` and all the pins stay green.
   `rook06_realpath_live`: 1 passed / 0 failed.

## What's left (the next session's named next step)

The 00 sister dies at tick 17230 at (8.50, 42.08, −37.17) — 23 hits for 100, 8 return-fire
fights — in the strip→descent lane. The pick census (in the session's traces, the fn prints it)
shows the exact remaining problem: the descent's deck guard (#31-class, y 47.0, weapon 0x25 —
the 45.0-class hit, 69.0 in 2 hits at d 8 on run 2) stands OUTSIDE the muzzle-ray set from the
courtyard floor (rays0 at the strip for #11/#19/#20/#23 — the ledges block the downward/upward
muzzle lanes), and Bond's nav walk then passes his d-8 lane. The levers, in order:
1. **A stand spot whose rays clear the deck**: the strip approach's nav recovery actually walks
   the DECK (the strip arrive was (6.08, 48.10, −32.77) — y 48, level with the deck posts) —
   run `aimed_stand_the_mouth` right at that y-48 arrival (the current tier runs after the
   arrive but the census there still showed rays0 — the spot within the arrive radius matters;
   try the arrive point ±1 u east/west along the deck, the pick is ray-honest).
2. **The nav STUCK recovery cost**: "the strip approach nav 1" routes back east through door
   #59 and gives up (3000 ticks) before recovering over the deck — a direct walked lane from
   the mouth (8.4, −12) down the courtyard's west edge (the recovery's own path: (6.09, 48.19,
   −32.71) → (4.67, 45.62, −34.80)) would cut ~3000 exposed ticks out of every run.
3. Then the descent end + the tunnel/gallery phases with the tier already in place.

## Verification (this session, on the final pushed tree)

| Binary | Result |
| --- | --- |
| `walkreal_07000006_live` (whole, 7 tests) | 6 passed, 1 failed — the failure IS the 00 sister (the descent lane, above) |
| `rook06_realpath_live` | 1 passed, 0 failed |
| 00 sister iterations | 4 measured runs on the merged tree (the cut run, the balcony-stand run, the strip-split run, the final tree), each rebuilt first; the ledgers quoted above are in the session logs (not committed) |

The full suite did not fit the window (the re-measure + the tier build + the four measured runs
consumed it); the 06 + rook06 binaries are the lesson-10 set for the touched file (test-only —
no `crates/*/src/` change; the branch's diff vs the base branch is the 06 test file, WORKLOG,
this report and the merge).

## Live checks for the main agent

1. From a fresh 00 Agent boot of 07000006: does the courtyard ch-55 alarm fire during the
   guardroom fights with ch 57 ON (the merged tree's door-route guards crossing the volume)?
   The port's ledger says #20 fired at frames 10563-10639 with the cut done — a PCSX2 watch on
   ch 55 + the spawner #735's spawn tick settles whether the port's volume/alarm chain matches.
2. The deck guard (y 47, weapon 0x25, the tunnel-deck post near (−1.8, 47.0, −46.8)): can a
   standing courtyard-floor Bond land an AIMED shot on him at d 12-16 (the muzzle-ray question
   the port's strict ray set answers NO to)? One PCSX2 aimed shot from (4.5, 42.0, −40) at the
   post pins whether the port's muzzle stand-in is over-strict or the game's own lane matches.

## Usage

Not visible to me: this environment exposes no token, cost or context-window figures for the session.
