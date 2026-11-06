# Aim and flight: the node pass rule, and honest fusebox tests

Branch from `origin/s/aim-object-flags`, push to `s/aim-node-pass`. Read its report.

1. `Door.bullet_pass` works by accident (0xdb & 0x60). Verify the 08 shutter leaves' glist nodes carry the 0x40 flag in the port's shape data, replace the stand-in with `Door.f0`, and let the node rule (HITDATA+0x51 bit 1 [M 0x12cea0-0x12cea8], `penetration_roll_node`) carry the shutters.
2. Land the mask gate and the cel-linked doors. Live [L]: the 08 rocket's aim ray hits class 0x1c object at (139.562, 50.792, -96.706), and the round flies (139.56, -95.472) to (139.568, -100.472) with that object in its hit chain.
3. The two 14 fusebox tests teleport in with the door closed. Rewrite them to walk the real route so the leaf stands open, as a player's would. Keep what they assert.

Must end green: rocket_stop_plane08_live, rocket_glass08_live, rocket_origin_live, solar_panel_hits_live, fusebox_geometry_07000014_live, interactables_07000014_live. Then report walkreal 08, 0c, 14, 16, 1b. Do not loosen a pin tagged [L]. Run each binary whole and quote results.

---
claimed-by: sandbox3 at 2026-11-06T01:06Z
