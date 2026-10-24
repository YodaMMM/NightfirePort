# level-0a-office-hatch-11 — the three wall-stop candidates are settled: the flags chain is vertical-only, Jointy is obj-vs-obj, the tail's radius is real — the stop's liveness now hangs on one live read (H50 during play)

## What was asked

Find the game's wall-stop [M] among hatch-10's ranked candidates — (a) the obj+0xdc flag word's
velocity-side use (`Player_CollisionHandler` 0x1949a8:154-165), (b) `Collide_Jointy` 0x1ec180,
(c) the tail's radius argument (`param_1[8]+0xc`) — port the one the decompile shows, with tests
for both requirements (01/02/03 walks green, the 0a glide/corner pins), and un-ignore
`the_west_hall_glide_height_matches_the_game`, `the_corner_crosses_like_the_game`,
`walk_office_floor_qworm_keyboard_hatch` as they go green.

## The settlement (all [M], decompile + raw-word disassembly of ACTION.ELF)

**(a) The flags chain is the VERTICAL stop, not the wall stop.** The word is `*(obj+0xdc)+0x60`.
`Player_CollisionHandler` 0x1949a8 clears bits 0xb8 (:27-28), then — from the first hit-list
record with +0x51 bit 1 clear (:110-152, :154-165) — sets bit 0x10 (a record exists) and bit 0x20
/ 0x8 by `hitpoint.y − BL+0x760.y` (A-point) sign. Consumption: bit 0x8 → `BL+0x14 = 0` and
`Vec_Zero(BL+0x50)` (:168-175), and the same pair in `Player_Collision` 0x194df0:124-127 gated on
`BL+0x110 >= 0.5` (the ground-normal-Y from `Player_FeetOnPoint`). `BL+0x10` is the walk's
local-frame displacement vector (`Player_Update` 0x193600:284-295 integrates
`pos += objmat·BL+0x10`; `Player_Move` 0x191dd0 writes only +0x10 = forward and +0x18 = strafe),
so `BL+0x14` is its vertical component: **the stop kills vertical drift + the gravity
accumulator `BL+0x50`. It never touches the horizontal components.** Who sets bit 0x8:
`Player_FeetOnPoint` 0x194188:176-185 — set when the scan's `HITTEST+0x9c != 0` OR the downward
feet ray hit (:99-100 — `Collide_RayIntersect(pos, pos−up·(cc+0.6), …, 0xc)`); the handler's
hitpoint path sets it too. And the hit list is NOT empty on plain walls (see (c)): the miss
records survive, with hitpoint (0,0,0) — below A — so **bit 0x8 and the vertical stop fire on
every tick near any world box** [M 0x1949a8:154-175]. The horizontal input-kill bit 0x1 is set
only by weapon-anim code (`Player_SetWeaponAnimObj` 0x1a2e38, `Player_Weapon` 0x1a24c8) — not by
collision.

**(b) `Collide_Jointy` is obj-vs-obj only.** Its call site `Collide_Intersect`
0x1ebaa8:100-105 sits in the `record+0x54 == 0` branch (an OBJ record) and dereferences
`record+0x58` (the other object) before the gate; the static-world path is the `else`
(:141+). It cannot be the static wall stop. (It remains the lift-cab ride path, and the ride
evidence stands.)

**(c) The tail's radius is a real radius — and the miss records survive.** The tail
(`Intersect_CylGeom` 0x109da0) works on a 0xa0-byte stack copy of the HITTEST, built at
`Collide_Intersect` 0x1ebaa8:157 (`amemcpy`, call 0x1ebc90; destination sp+0x50, cached in $s0
at 0x1ebc8c — disassembled). Its parameters, at instruction level:
- A/B slots localcopy+0x20/+0x30 are OVERWRITTEN after the copy by the cel-space transform
  (RotPreTransVec outputs sp+0x70/sp+0x80, calls 0x1ebe30/0x1ebe54) or, when the scanner matrix
  is identity (static world, `Ident != 0`), by the raw `lq 0x30($s2)` / `sq 0x80($sp)` pair
  (0x7a420030 / 0x7fa20080 at 0x1ebe44/0x1ebe4c). For static world the matrix is identity, so
  the distinction is inert there.
- **localcopy+0x50 — the guard's reference point — is nothing but the amemcpy copy of
  `HITTEST+0x50` = BL+0x790** (nothing writes it before the `jal Intersect_CylGeom` at
  0x1ebe68). Hatch-10's dead-accumulator reading of the reference is CONFIRMED at the
  instruction level; for dynamic objects the copy is transformed into the object's frame
  (:160-168), which is exactly how a lift cab keeps the tail alive inside itself while the
  world-space +0x50 accumulates the ride vector — matching the [L] 05 measurement
  (BL+0x790 = (1.2079, 9.0838, −14.4486) ≈ the descent).
- **The radius word `param_1[8]+0xc` = localcopy+0x8c = `BL+0x7cc`** — per-tick seeded by
  `Player_Collision` [M 0x194df0:204/231/255/284/316/341/379]: **0.275 for the walk modes
  (case 0/6), 0.55 otherwise.** Not a pointer-as-float. The walk capsule's bottom sphere B
  (pos − up·0.4847) sits exactly 0.275 above the feet-line (cc = 0.7597, [M 0x194188:88-92]):
  **the capsule rests tangent to the floor at exactly one radius** — the 0.275 reading is
  structurally confirmed.
- **On a miss, `Intersect_CylGeom` writes record+0xc = 2.5e7 (0x4cbebc20, :587), not 1e8** —
  and `Collide_Update` 0x1ea398:125 frees only `== 1e+08` records. **So the walk's hit list is
  NON-EMPTY near world boxes even with a dead tail** — every nearby box yields a record (the
  broad phase in `Collide_Pick` 0x1eb320 allocates one per box, kind-0x800 branch :224-241),
  miss records sort last (2.5e7), hit records first (the real distance). This resolves
  hatch-10's live check 1's second half: `obj+0xd0` will NOT read 0 on a wall press — read
  `record+0xc` and `BL+0x790` instead.

**Consequence.** With the tail's only horizontal response gated on `|q − BL+0x790| < 0.275`
[M 0x109da0:447], and BL+0x790 born (0,0,0) at a fresh 0a boot [L ee_start_0700000a], the tail
cannot fire until BL+0x790 comes within 0.275 of a triangle — yet the [L] fresh-boot glide HOLDS
the z 16.29 line, and only the tail (handler :136-140, `pos += BL+0x750`) or an input kill
(none in collision) can stop the walk horizontally. The two [L] facts fit one model: **H50
self-tracks once seeded.** Each landed tick adds `DeltaP` into +0x50 [M 0x1ebaa8:353-373], and
`DeltaP = (q−H50)/|q−H50|·(radius−|q−H50|)` [M 0x109da0:448-457] is a spring from H50 toward the
contact — so if BL+0x790 ever comes within one radius of ANY touched triangle (the entry-lift
cab is the obvious seeder on 0a; inside the cab the transform keeps it in-frame), it then hops
from contact to contact and follows Bond's feet everywhere, making the tail a live capsule
push-out — including the 0.011-inset glide-line equilibrium. The alternative (the stop is
outside the whole BL+0x740 scan) has no remaining candidate in the walk path. **This is a model,
not a proof: it predicts BL+0x790 ≈ Bond's feet position (within ~0.3) during real 0a play. One
live read settles it.**

## What I ran

- Setup had to fight the sandbox: root fs filled at 100% (freed 6.1 GB — the previous
  sandbox's stale clone at ~/my-project/NightfirePort, its tip verified identical to
  origin/s/cutscene-side-strips before deletion); ALSA built from a /tmp prefix via
  PKG_CONFIG_PATH plus the repo's `NF_ALSA_LIB_DIR` link hook (with
  `touch crates/nf-audio/build.rs` + `cargo clean -p nf-audio -p alsa-sys` to invalidate the
  cached build-script output).
- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live -- --test-threads=1`
  did NOT complete: the link step raced the background suite for the target-dir lock and the
  sandbox clock ran out during the repeated restarts. **No test result this round; no code was
  changed, so the tree is exactly origin/master.** The suite (`tools/test/suite.py`) was
  restarted four times over the ALSA/PATH issues and did not reach a total line before the
  deadline — FAILED/TIMEOUT lines: none observed, but treat that as no evidence.

## Why the three tests stay `#[ignore]` (no code landed)

Porting the tail as decompiled (with the H50 accumulator) without knowing H50's live value is a
coin-flip on the 01/02/03 walks (hatch-10's ghost experiment already showed the walks fail when
the port's push goes away). The flags chain is vertical-only and cannot replace the port's
horizontal stop. No [S] may enter pass/fail, so nothing can be un-ignored this round. No
decompile-backed test was weakened; nothing was committed except this report and the worklog.

## What's left (the next round's shape)

1. **The one live read that decides the port** (below). If H50 tracks (within ~0.3 of Bond's
   feet mid-play): port the tail exactly — H50 accumulator (born zero, += DeltaP per landed
   hit), radius from BL+0x7cc (0.275 walk), the branch-A gate `0 < dot(q−H50, n)`, the
   miss-record survival semantics, and the handler's `pos += BL+0x750` + flags chain — then find
   the seed event on the boot path (the entry-lift cab is the prime suspect; check whether the
   cab's ride lands the first +0x50 add). If H50 reads ≈ (0,0,0) mid-play while the glide still
   holds, the stop is outside the entire BL+0x740 scan and the hunt moves to the control layer
   (the `control_funcs` around `Collide_Update` 0x1ea398:54-70).
2. Then un-ignore the three tests as they go green and drop the env switches.
3. Re-run the walk battery and the fast suite on a sandbox where the ALSA link env is set from
   the first build (both PKG_CONFIG_PATH and NF_ALSA_LIB_DIR, plus the build-script cache
   invalidation) so no target-dir race recurs.

## Live checks for the main agent

Fresh boot of 0700000a (save-state resume skips the boot path):

1. **The decisive one — is H50 tracking?** Mid-walk in the 0a office (Bond walking), read
   BL+0x790 (the obj is `glb_players[0]` @ 0x2d88e0 → +0xE0 → BL). Prediction of the
   self-tracking model: within ~0.3 of Bond's position at ground level (≈ pos − up·0.48);
   the dead-tail model: (0,0,0) or the entry-lift's net ride vector, far from Bond.
2. **Watch a wall press through the handler.** With Bond pressed into the doorway jamb
   (x −8.0 → −8.4 at z 16.29), single-step gdbwatch the WRITE to obj+0x30 for 10 ticks and
   record each write's exporting address. Expected under the model: `Player_CollisionHandler`
   0x1949a8:136-140 (the `pos += BL+0x750` store at the +0x30 write) firing each tick with a
   ≈ +0.011 z-step — the glide equilibrium; plus `Player_Update` 0x193600:284-295 (the walk
   integration).
3. **The seed event.** Right after the entry lift lands (before any walk input), read BL+0x790:
   if it is already non-zero and near Bond, the cab seeded it — grep the boot path for the
   first landed-tail tick (the cab's `Ident==0` transform path at 0x1ebaa8:160-168 is the
   mechanism).

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
