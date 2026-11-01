# player-trigger-contact — session report (2026-11-01)

Branch `s/player-trigger-contact` (from `origin/master` 6042ee53). The sandbox window was ~2 h; this
session mapped the game's real trigger-contact chain with addresses, fixed the one confirmed stale
comment, and found the blocker that any real-rule port must clear first. The port of the real rule
itself is the continuation's job — the open item is named precisely below.

## What the game does (verified this session)

The chain the task cites, read in the decompile:

1. `control_movement_object_handler` 0x1336f8:54-68 [M]: after `Trigger_Calc` and the prev/hold
   snapshots, every object in `DynamicObjList` with `obj+0xFA & 6` gets `Mat_GetDir(obj+0x60 →
   obj+0x90)` and its class control function (`control_funcs[obj+0xFF]`) — the player's movement and
   its HITTEST stamps run here, every tick.
2. `Collide_Update` 0x1ea398 [M]: first clears every object's `+0xD0` contact list back to the pool
   (:25-53), then per object with a `+0xD4` HITTEST and a candidate count (`HITTEST+0x94`): `Collide_Pick`
   0x1eb320 gathers the candidates, and `Collide_Intersect` 0x1ebaa8 runs the real test — in the
   candidate object's local space (`Mat_Inverse` + `RotPreTransVec` of the HITTEST's cap points,
   :277-295) it calls `Intersect_CylGeom` 0x109da0 (:296) or `Intersect_CylTriGeom` 0x1e9a70 (:256).
   Hits sort by `+0xc` (distance, :119-141) and each surviving hit is MIRRORED onto the other
   object's `+0xD0` as a contact node (:160-215: `+0xc` = the hit's param, `[2]` = the owner's
   matrix word, `[0x10..0x13]` = hit+0x40.., `[4..7]` = hit+0x10.., `[0xc..0xf]` = hit+0x30.., the
   `+0x50/+0x52` flag bytes copied, `[0x16]` = the contact owner).
3. `Trigger_Hit` 0x1b1fc8:27-47 [M]: walks the trigger's `+0xD0` list and classifies by the contact
   owner's class byte `obj+0xFF` — 3 → bit 8 (the player), 2 → 0x10 (a drone), 5 → 0x2 (an object),
   0x4A → 0x10 — then the trigger's mask/filter chain decides (:55-122).
4. The player's HITTEST capsules per stance — `Player_Collision` 0x194df0 writes them into
   `BLData+0x760/0x770/0x7cc` (the cap centres + the radius) [M]: case 0/6 (walk): radius **0.275**,
   caps at `up·0.55` and `up·(0.275 − cc)` (:204-227); case 3 (crouch): radius **0.55**, caps
   `up·(0.1 − (cc − 0.55))` and `up·(0.45 − cc)` (:228-253); case 5: radius 0.55, caps `up·0.275` /
   `up·(2.1 − cc)` (:254-279); case 0xB (non-class-6 mount) and the default arm: radius 0.55, caps
   `up·0.275` / `up·(0.55 − cc)` (:280-338). `cc` = `AnimObject+0xCC` (obj+0xDC → +0xCC, read once
   at :86) — live-varying: standing 1.0347183 [L], crouched ≈ 0.55 [L trace-derived, ≈ the crouch
   capsule radius, CROUCH_ROOT_HEIGHT].

The port already carries all of this: `player.rs::pick_arm` (the stance arms above) and the
`DoorMover` collide path (door.rs) use the pick capsule; `drone.rs:18081-18091`'s trigger toucher
still uses the measured stand-in `Toucher::player([x, y − 0.6, z])` — a capsule centred
`[ref−0.05, ref+1.26]`, r 0.55, whose lowest surface point is `ref − 0.6` (the reach measured to the
`[0.557, 0.575]`-below-reference window, drone.rs:18046-18070).

## The blocker for the real-rule port (found this session)

The naive swap (use the case-0/6 pick capsule as the trigger toucher) is **contradicted by the
disc**: the standing capsule's bottom is `pos − cc = feet + 0.051`, which lies BELOW 07000005's
ch-0x70 park quad (a flat quad at y 37.7418 = feet + 0.46 [D script stream 6, key 8]) — so the
game-accurate capsule fires the park plate while Bond stands on the truck bed. The disc does not
fire it standing [D fix-truck's trace]; `fix_truck_live::standing_ride_never_touches_the_parking_plate`
pins the port's agreement (re-ran green this session). This is exactly why master's earlier
game-accurate attempt was REVERTED (drone.rs:18050-18057). So the pick capsule alone is not the
game's trigger-contact discriminator: a gate somewhere in the collide path keeps the standing
contact off the trigger's `+0xD0`. The mirror gate's inputs are now NAMED (this session's second
pass):

- **`hit+0x50` = the touched triangle's surface class byte** — `Intersect_CylGeom` 0x109da0:568 [M]
  copies it from the glist's surface-property table (`piVar2[2]`, the glist's class base) indexed by
  the triangle's surface index. Per-triangle data, not per-object.
- **`hit+0x51` ORs in the caller object's `obj+0x9a` byte** — `Collide_Intersect` 0x1ebaa8:178 [M]
  `hit+0x51 |= param_3` where param_3 = `*(u16*)(obj+0x9a)` passed by `Collide_Update` 0x1ea398:94;
  `Collide_Update` also sets bit 1 of it on every mirrored contact (0x1ea398:190).
- The mirror gate 0x1ea398:216-229 [M] therefore reads: `hit+0x51 bit 1` (the caller's `+0x9a` bit 1)
  OR `hit+0x50 bit 6` (the surface class's bit 6), with `hit+0x50 bit 7` adding a `Rand_Rand(0x3f)`
  random arm and the caller's `obj+0x9a & 0x20` arm forcing a mirror unless the surface class is
  0x10. A contact whose surface class lacks bit 6 and whose caller lacks `+0x9a` bit 1 NEVER mirrors
  to the other object's `+0xD0` — i.e. **whether a trigger sees a body is decided by the touched
  triangle's surface class byte (bit 6) plus the mover's `obj+0x9a` bits**, per hit.
- The remaining unknown (the continuation's first read): what the plate triangles' surface class
  bytes are on the disc (a dump of the glist surface table for the ch-0x70 park quad and #277), and
  what the VU microcode inside `Intersect_CylGeom` (the C at 0x109da0:421/536 shows only the
  `_cfc2` flag checks; the math is VU0) contributes — whether a triangle fully inside the cylinder
  (a flat quad under the axis) produces a hit at all, and with which surface index.

## The three known cases

- **0700004a reveal plate #277** (ch 0x4d): TouchOnce volume (0.90–5.18, −9.87..−7.65, −5.74..−1.01)
  [D] — 2.22 tall, and its XZ is offset from the board point (the spawn lands at z ∈ (−6, −3), the
  plate spans z −5.74..−1.01). The miss is XZ, not vertical reach: no body shape fires it from the
  board point; the walkreal walk (the real player's first steps north, walkreal_0700004a_live:92-105)
  crosses the plate's XZ and is the real input path, not a dodge. Re-ran green (2 passed). Kept.
- **07000005 spawn reveal plate ch 0x45**: TouchOnce #36 at (105.94, 24.41, 108.14), out 0x45 [D].
  It FIRES on the boot tick under the −0.6 stand-in (the plate sits 0.557 below the boot reference,
  inside the stand-in's window); walkreal_07000005_live:100 asserts the fire and passed. The comment
  at walkreal_07000005_live.rs:65-67 still claimed the plate "sits below the toucher stand-in's
  reach" — STALE (written pre-−0.6); fixed this session.
- **07000002 bunker plates #385/386/387** (outs 0x7E/0x7F/0x80): locked by the ambush (lock 0x81
  [D]) in every real route; walkreal_07000002_live pins them OFF (:695, :976 — re-ran green). Their
  world extents are not yet dumped from the disc (docs/levels/07000002.md has no coordinates), so
  the "0x80 below the floor" depth claim is unverified, and a firing pin needs either a pre-ambush
  route or an unlocked state. Left for the continuation together with the real rule.

WORKLOG: `docs/WORKLOG.md` no longer exists on master (dropped as a process file in batch-1101j), so
no entry could be appended; this report is the session record.

## What ran (totals quoted)

All with `NF_SILENT=1 cargo test --offline --release`:

- **nf-game**: 100 binaries, **1028 passed, 0 failed, 9 ignored**.
- **nf-formats + nf-audio**: 10 binaries, **136 passed, 3 failed, 0 ignored** — the 3 are exactly the
  `disc_golden` tests the slim ISO is allowed to fail.
- **nf-view, the walkreal family (mission routes)**: 23 binaries — walkreal_07000001/02/03/04/05/06/
  07/08/09/0a/0b/0d/11/12/13/14/15_live, walkreal_0700004a_live, fix_truck_live, mayhew_handover_live —
  **79 passed, 0 failed, 4 ignored**. (The remaining ~82 walkreal probe binaries and the rest of
  nf-view's suite did not fit the session window; see "What's left".)
- The full four-crate run of `tools/test/suite.py` (its total line) is the continuation's first item;
  this session's sandbox kills background processes between tool calls and the fresh target's build
  plus ~950 binaries does not fit the remaining window with the foreground-only fallback.

## What's left (the continuation)

1. Dump the glist surface-class bytes for the ch-0x70 park quad (07000005) and #277 (0700004a) —
   do they carry bit 6? — and read the VU0 microcode behind `Intersect_CylGeom`'s flag checks
   (0x109da0:421/536) for the inside-the-cylinder case.
2. Port the real rule into the toucher path (per-stance `pick_arm` capsules + the gate), keeping
   every pinned behaviour green — fix_truck_live's standing sister is the arbiter.
3. Dump 07000002's #385-387 extents; add the firing pin if a real route can reach them unlocked.
4. Full `tools/test/suite.py` run on the final tree; read every red.

## Live checks for the main agent

1. `fieldprobe` on 07000005, Bond standing parked on the truck bed (pre-crouch): read the player
   obj's `BLData+0x760`, `+0x770` (cap centres) and `+0x7cc` (radius) — expected r 0.275, caps
   `pos+0.55` and `pos+0.275−cc` [M 0x194df0:204-227]; and `AnimObject+0xCC` — expected ≈ 1.0347 [L].
2. `gdbwatch` the ch-0x70 TouchOnce object's `+0xD0` during the standing park: expected EMPTY (no
   contact node written) — vs the crouched park: expected one node whose `[0x16]`/owner is the
   player object (class byte 3). This decides whether the standing gate lives in the collide path
   (no node) or in `Trigger_Hit`.
3. `gdbwatch` the HITDATA `+0x50/+0x51` flag bytes and the `Rand_Rand(0x3f)` call count for the
   trigger-candidate hits during one standing-park tick (Collide_Update 0x1ea398:216-229's gate
   inputs).
4. `fieldprobe` the bunker TouchOnce #385/386/387 world matrices (`obj+0x90`) on 07000002 — their y
   extents vs the corridor floor (the "0x80 below the floor" depth claim is currently unverified).

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures for the session.

## ntfy failed

All three fallbacks failed (2026-11-01 ~23:35 UTC); exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "player-trigger-contact: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'player-trigger-contact: …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "player-trigger-contact: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, killed at the 30 s tool timeout (no connection).

github.com pushes from the same sandbox succeed, so the block is specific to ntfy.sh egress.
