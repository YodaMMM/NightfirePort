# Report: cutscene-handback-census

Branch `s/cutscene-handback-census` (based on master 08f06953 + the kind31-frame-2/3 rule, merged as origin/s/cutscene-kind31-frame-3). Extends cutscene-kind31-frame-2's kind-0x31 census to **every SP script's event-6 hand-back on every on-foot level**, computes the position the port hands Bond back at, and checks it against the level's own floor geometry.

## The census

`crates/nf-game/tests/nis_handback_census.rs` — per on-foot level archive `0700*.bin` [D FILES.BIN], every kind-0xd9 SP placement [M `Player::from_placement` params g0/g1, base = `View_RotTransMatrix` 0x1db9a8 → 0x1344e0], every script with an event-6: the script runs through the REAL port runtime (`Script::load` at the placement base → one `Script::update` per 60 Hz tick → `Script::free`), and the `Request::PlayerEnable` snapshot is checked against the level's static collision (`CollisionWorld::build` of the main map, `floor_below` within 2.5 units — the game settles a re-enabled player onto it).

**Result: 26 hand-back rows over 14 levels, 12 fallbacks, 0 voids** (the two 4a rows ride the [L]-pinned landing-pad exception below). Every snapshot position stands on a floor: 01's intro hand-back (-8.49, 6.48, -9.78) on floor 5.34, 14's capture marker (-55.49, 1.55, 42.02) on floor 0.0, 07's party scenes, 05's ledge scene, 08's gondola intro, 0b/0d/16/04 rows — all supported [D level collision]. The full table prints with `--nocapture` (script lib: 221 scripts from the packs).

## Findings

1. **The 4a landing scene is correct, not a void** — `0x060006cf` (placements #273/#289) hands back at (1.80, -54.13, -3.45), which has no STATIC floor within 2.5: the floor there is the lift platform, the SP #289 placement's own dynamic object [D placement base (0, -53.534, -3.377)]. The position matches level-4a-lift-4's live measurement exactly — the game re-enables Bond at the hash-0 kind-0x19 marker key's own x/z [L fresh boot; D walkreal_probe_l3 key t575 = (1.79945, -54.12811, -3.45340)] — so the census pins it as a named exception (position equality, 0.05 tolerance) instead of a floor check. `fix_finale_live` (the same scene end-to-end) stays 4/4 green.
2. **A real divergence, documented not pinned: kind-2 (anim) prev streams.** The game's event-6 gate admits prev-stream kinds {1, 2} and snapshots the object's +0x90 either way [M `Script_EventHandler` 0x1d94c0:73-84 `*(byte*)(record+0x24) - 1 < 2`, :125-134 `Mat_Copy(iVar5+0x90 → si+0x40)`]; an anim object's +0x90 is live — `AnimObjectUpdate` writes it through `Control_GetObjMatrix` every frame [M 0x1befd8:290 → 0x134210 `View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos)`]. The port's event-6 arm reads only `Obj::Entity` and falls back to the player's own position for a kind-2 prev stream. Four census rows hit this: 07000002's 0x0600007f (s3→s2) and 0x06000084 (s3→s2), 07000003's 0x06000080 (s2→s1), 0700004a's 0x060006d3 (s4→s3) [D decode + NF_SCRIPTLOG event-time kinds]. The other eight fallbacks have prev kind 0 (stream never started: the game's `0 - 1` wraps unsigned and fails the same gate) — the port matches the game exactly there. The four kind-2 rows are left UNASSERTED (a port choice is never a hard assertion); fixing them needs an anim-root placement at event time, see What's left.
3. **0x06000074 (07000005 SP #71's script) exists nowhere on the disc** — no kind-7 entry under that name in any archive, no kind-0xC loadable record [D probe, `--ignored probe_missing_hash`]. The port's `Player::load` missing-script arm (state 5, deleted) is the correct behavior; the census prints it as not censusable.
4. The frame-2 rule holds across the whole disc: every snapshot row's script has its camera op at tick 0 (cam@0 in every table row), so all entity keys — and thus every hand-back — are keys-alone world-frame, and the census found no position that only composes correctly under the old per-class scopes.

## What changed

- `crates/nf-game/tests/nis_handback_census.rs` (new): the census + the not-in-pack probe; the hard assertion is only the geometry verdict ("no hand-back over void, except the [L]-pinned 4a landing pad at its measured position").
- `crates/nf-game/src/nis.rs`: one read-only accessor, `Script::stream_kind(i)` (the census reads the event-time prev-stream kind; `kill` clears kind bytes at free). No behavior change.

## Verification

What ran (foreground chunks; this environment kills background jobs between tool calls, so the full suite's ~866 binaries could not all fit the window — the run was prioritized):

- All three lib bins, release: **nf_formats 91 passed / 0 failed**, **nf_game 578 / 0 / 5 ignored** (includes every nis.rs unit test), **nf_view 96 / 0**.
- The cutscene/hand-back-sensitive integration set (14 bins, 2-way): **13 green** — nis_handback_census 1/1, nis_kind31_census 1/1, capture_placement_07000014_live 1/1, exit_aim_01_live 1/1, exit_aim_hint_live 2/2, interact_07000001_live 10/10, walkreal_07000001_live 3/3, walkreal_07000014_live 2/2 + 1 ign, walk_07000014_live rc=137 (**OOM kill, not a failure** — the runner's memory ceiling; the same walk passed on master's tree in prior sessions), fix_finale_live 4/4, fix_kimiko_handback_live 1/1, cutscene_side_strips_live 3/3, simplescript_live 2/2.
- Not run this window: the remaining ~845 integration binaries (nf-game's other 90+ bins, nf-view's long walks, nf-audio). No disc_golden/identity_disc failures were triggered (those bins were not among the ones run).

## What's left

- The kind-2 prev-stream fix [S suggested path]: carry the anim object's world matrix (the game's `Control_GetObjMatrix` product) on `Obj::Anim` — the port's `base: Mat4` field plus the anim root track already in the ride-pack work would supply it — and let the event-6 arm snapshot `Obj::Anim` too, then re-run this census: the four fallback rows become snapshots whose positions must land on floors. The census test is the safety net for that change.
- The full suite on the final tree (the prioritized subset above ran green; the rest is unchanged-from-master code paths except the one accessor).
- Live checks for maintainers: (1) 07000002/03 — after the 0x0600007f/0x06000084/0x06000080 cut-scenes end, where does Bond stand relative to the scene's anim actor (the kind-2 divergence predicts the game hands him back AT the actor's anim pose, the port where he was disabled)? A fresh-boot [L] of Bond's position at each scene's end would pin the game's rule for kind-2 prev streams. (2) 0700004a landing pad — already pinned [L level-4a-lift-4], re-confirm unchanged.

## Usage

not visible to me
