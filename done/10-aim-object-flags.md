# Aim and flight: the real object flag word

Merge `origin/master` into `origin/s/aim-object-stage`, push as `s/aim-object-flags`. Read its report. Blocker: the 14 security room's near door leaf stops the laser, because `Door.bullet_pass` reads placement flags as a stand-in for `obj+0xF0 = glist_record+0x44 | create_param` [M Control_CreateObjEx 0x1343c8:36]. Parse glist+0x44 from the disc and use the true word for both the flight's pass rule (`andi 0x60` at 0x12cf18) and `Collide_PickObj`'s gate (`obj+0xF0 & mask`, mask 0x20 plus 0x1020/0x40/0x10 by hittest flags [M 0x1eae80]). The panels read f0 0x81 live, the shutter pieces 0xc1, debris 0xa8a0 [L audits are in the task above: 0700001b].

Then: fusebox_geometry_07000014_live, interactables_07000014_live, and every walkreal the earlier reports list red (08, 0c, 14, 16, 1b with player firing spots). Report each. Do not loosen a pin tagged [L].

---
claimed-by: sandbox3 at 2026-11-06T00:10Z

---
## Report (done, 2026-11-06T01:05Z)

