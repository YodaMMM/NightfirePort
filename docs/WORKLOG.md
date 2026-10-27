
placement-matrix-classes — started (2026-10-27 ~13:45 UTC)

- Goal: every placement class builds its object matrix the way the game does.
  0xd9 ScriptPlayer placements go through control_create_object 0x1344e0's param_4==0
  arm = View_RotTransMatrix 0x1e5168 (port: placement::view_rot_trans_matrix), not
  RotMatrix 0x1e2368. Pin each class's creator matrix builder [M], switch the port
  where it differs, census non-zero rx/rz placements per class [D].
