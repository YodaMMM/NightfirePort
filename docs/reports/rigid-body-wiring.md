# rigid-body-wiring — the kind-0xD8 bodies are sim objects: created, ticked, blast-armed, matrix exposed; both live tests green

## What's done

1. **Creation** (`Rbodies::create`, crates/nf-game/src/rigid_body.rs): every main-map
   kind-0xD8 placement becomes a sim body per `RB_Create` 0x1d6bf0 [M] — type =
   rec+0x34 (`extra[4]` low half), mass = rec+0x2c (0 → 3.0, sign-bit → the raw word as
   f32, else i32 as f32) [M :60-64], the moments from the glist extents
   (`RB_CuboidMomentsOfInertia`), the vert table by type (BOX_VERTS 0x2d8890 for type 0,
   SPHERE_VERTS 0x2d6200 otherwise) [M :74-78], +0x9c = |min.y|, the +0xAA/+0x13c tail
   (0x101 default kept for type 0; 0 with min(Δx,Δz) for 2/4; 0 with the sqrt-half-diagonal
   for the rest — the census barrels read type 0xFFFF) [M :80-96], +0x154/+0x158 from
   rec+0x38/0x3c [M :100-104], obj+0xFA |= 0x44 [M :56]. Wired into `Drones::attach_level_id`
   next to the other parsemap-epilogue creators. `st.pos`/`st.trans` start at the placement
   translation — the at-rest state is the static draw (the drawn translation follows rb+0x00
   [M EulerEvolve 0x1d796c]; the literal create-time store is a live check below).
2. **Tick** (the control walk, drone.rs kind 7): `rb_update` 0x1d6e38 per body at the
   game's per-kind dispatch position ({RB_Update, RB_CollisionHandler} at 0x26f9c8/0x26f9cc
   [M ELF scan]; the runtime order is the object-list creation order, which the
   placement-keyed sort preserves). The heat-puff gate passes nothing — the port has no
   per-object room cel yet [S] — so a resting body never draws the shared RNG (the game's
   draw is gate-gated [M 0x1d6e88-0x1d6eb0]); wind is zero (the `Env_GetWindVel` census is
   the remainder's open item [S]); obj+0xFA |= 0x20 per tick [M the update tail].
3. **Blasts** (`spawn_blast`): the game's explode sweep finds the objects
   (`Collide_SphereIntersect` 0x17c518:67) and the falloff is the shared exact form
   (:78-81 — the port's `mp::combat::blast_falloff`); the class-5 hit record arms the body's
   hit list and `RB_CollisionHandler` 0x1d7a80 [M] dispatches — type 0 →
   `RB_RestingContact` 0x1d7168's walk (the CalcTorque gate `*(short*)(rb+0xAA) != 0x400`
   [M :0x1d71f8], class 5 → the impulse arm `rb_explosion_hit_arm` [M 0x1d7238-0x1d72e4],
   class 3/0x1a skipped, else the position correction [M LAB_001d7298]), type != 0 →
   `RB_RestingContactRot` 0x1d73a0's walk (class 5 only, with the per-type torque-lane
   masks [M 0x1d7478-0x1d74c8]). The impulses land in +0x40/+0x50 and the NEXT tick's
   `RB_ApplyImpulses`/`RB_EulerEvolve` applies them (the game's Collide_Update is
   post-walk [M 0x1336f8:238]).
4. **`RB_CalcTorque` 0x1d7658 + `Vec_MakeOrthonormalBasis` 0x1e4b78 ported** [M, literal
   from the decompiles — the three basis dots against *(rb+0x00), the ±0.05 gates, the
   −0.01/−0.2 scales].
5. **Matrix exposure**: `RbObj::matrix()` composes obj+0x90 (the quat rows + the `st.trans`
   translation row); the bodies join `set_dynamic` as collision parts (id bit 43 + the body
   index) so bullets/Bond/LOS see them like every cel-linked object [M `Collide_PickObj`;
   RB_Create's build_LinkToRoom]. The scene draw swap is the separate task ("the scene pose
   is done separately") — the accessor is the deliverable.
6. **The hit-record construction is the documented [S] stand-in**: the game's
   explosion-object contact record (+0x30 vel, +0x40 point) is unported. The port's record:
   vel = the body→blast unit vector, point = the body's surface point toward the blast plus
   a fixed deterministic tangent. The tangent is REQUIRED by the game's own math: an
   axis-aligned point makes r ∥ impulse and `RB_CalcTorque`'s r×impulse reads ZERO for
   every sphere-sphere blast, while the game's gas-volume blast visibly tips barrels —
   the game's contact must be off-axis (live check below).

## How it was verified

- `crates/nf-view/tests/rigid_body_wiring_live.rs` (NF_ISO, 07000006 — a [D] barrel
  level), both green:
  - `resting_barrels_stay_bit_identical_over_the_sim` — after boot, 600 sim ticks leave
    every body's pos/omega/vel/ang_accel/accumulators/matrix/trans/s9c/s13c/flags
    bit-identical [M the type-0xFFFF arms: no linear integration, zero omega skips
    apply_omega; the heat gate closed].
  - `blast_next_to_a_barrel_changes_its_matrix_the_games_way` — a props-shape blast
    (radius 3, damage 200, off-centre) moves the barrel's matrix; the post state equals
    `rb_explosion_hit_arm` + one `rb_update` replayed on the pre-state with the SAME
    record the sweep built (the routing pin), and 600 further ticks stay finite.
- `nf-game --lib` **578 passed / 0 failed / 5 ignored** (-j1; under -j2 two sdk disc-census
  tests hit the sandbox's concurrent-ISO-open limit and fail — they pass in isolation and
  are untouched by this diff).
- **The full suite did NOT run in this window** — the sandbox was reset (repo/ISO/toolchain
  re-fetched from scratch consumed ~40 min) and the remaining time went to the wiring and
  its tests. The next window must run `tools/test/suite.py` (chunked — background runners
  die at the tool-call boundary in these sandboxes) and watch the walkreal/door/mp families
  for the new collision parts (the barrels are now solid — that is the game's truth [M], but
  any walk path that leaned on them being ghosts will surface there).

## What's left

1. The full suite on this tree (above).
2. The bullets' route through the hit list: the bullet-vs-RB contact should arm the same
   walk (the record from the bullet's velocity/point) — the parts are in place, the arming
   site (`update_player_bullets`'s part-id dispatch) is not wired yet.
3. `Collide_CylinderIntersect` 0x1ecc00 + `RB_RestingContactRot`'s cylinder interior (the
   +0xD0/+0xE0/+0x100 test points, the uprighting tail with the rb+0xC0 normal) — the
   remainder report's item 2, still open; until then the type-0xFFFF barrels have no
   ground-contact response.
4. `RB_EulerEvolve`'s type != 0 up-righting tail (0x1d79f4+, incl. the +0x40 linear
   response for non-zero types) — remainder item 3.
5. The per-object cel + `Env_GetWindVel` census (the heat puff and the wind) — remainder
   item 4; the sim passes None/zero today [S documented].
6. The scene draw swap onto `RbObj::matrix()` (the visual work — the main agent renders).

## Live checks for the main agent

1. `RB_Create`'s lost stores: at create, watch `rb+0x00`/`rb+0x20` (the position/velocity
   slots) and `obj+0xC0` — the decomp's prologue is lost; the port assumes the position
   slot starts at the placement translation (the rest-state identity argument). Address
   0x1d6bf0, write watch, expect the placement pos.
2. The explosion contact record: blast a barrel and watch the class-5 hit's +0x30/+0x40 as
   `RB_RestingContactRot` reads them — is +0x40 on the sphere-sphere axis (→ zero torque,
   the port's stand-in wrong) or off-axis (→ the gas volume, the stand-in right)? The
   sound's volume arg (*(rb+0x158)) is also visible there.
3. Render check after the draw swap: `--bond` at a 07000006 barrel before/after a satchel
   blast — the barrel should tip about the off-axis contact, not slide.

## Usage

not visible to me — this session's driver surface shows no token, cost or context-window
figures (the IM gateway metadata carries only session/chat/trace ids).
