
placement-matrix-classes — started (2026-10-27 ~13:45 UTC)

- Goal: every placement class builds its object matrix the way the game does.
  0xd9 ScriptPlayer placements go through control_create_object 0x1344e0's param_4==0
  arm = View_RotTransMatrix 0x1e5168 (port: placement::view_rot_trans_matrix), not
  RotMatrix 0x1e2368. Pin each class's creator matrix builder [M], switch the port
  where it differs, census non-zero rx/rz placements per class [D].

placement-matrix-classes — the per-class matrix builders are pinned and switched (2026-10-27)

- Catalogued every `parsemap_create_dynamic_objects` 0x1d04f0 case: every creator that reaches
  `control_create_object` 0x1344e0 passes the quaternion arg as literal 0 (all 91 call sites in
  ACTION.ELF), so the object matrix is always View_RotTransMatrix 0x1e5168 — the Y·Z·X
  composition — while the 0x8000 static path (parseentity_transform_bounding_box 0x1d0fb0:28-29)
  and MP_RegisterMPObject 0x185990:64-65 use RotMatrix 0x1e2368 (X·Y·Z), View_AddSkyObj's sky
  gets RotMatrix at draw time 0x1e5ad0:184,194, GunImp 0x131f20:66-69 and the 0xf2 emitter
  (0x1d04f0:474-478) use Quat_QuatToMat on the placement quaternion, and Drone_Create
  0x136be0/DroneSpawner 0x1378d0 build identity (NULL pos/rot). The class ids differ from the
  task brief: doors are 0xdb, ladders 0x22, sky 0x2a.
- Switched the port: `placement::object_matrix_for_class` (the per-class predicate with the full
  [M] catalog) in level.rs `instances()`, the cloud arm in `sky_instances()`, and the per-class
  sites in nf-game (triggers, breakables+props::bounds, sensors, PCQ spheres, mines, fusebox/
  switch/GT script players, shooters, dynobjs, copter, searchlight, pickups, hurt, doors).
  Statics (0x19 etc.), drones and the exceptions keep RotMatrix; yaw-only dynamic rows move by
  at most one cosine ulp (entry [0][2]/[2][0]) and a +0.0/−0.0 sign at [1][0] — pinned in tests.
- Disc census [D] `placement_rot_census.rs`: 99 levels / 2230 maps; the pitched reach per class
  (sensors 29/25, coronas 57/36, pickups 41/42, SS 20/21, grapple 19/19, dynobj 23/23,
  emitters 16/52, doors 0/447 all yaw-only, statics 42/4766).
- Verified: nf-formats unit tests (closed-form View/RotMatrix rows, the class predicate, the
  yaw-only ulp), nf-game 95 binaries 933 passed / 0 failed, 47 placement-sensitive nf-view
  binaries 96 passed / 0 failed (suite.py), the 3 disc_golden exceptions per the slim ISO.
