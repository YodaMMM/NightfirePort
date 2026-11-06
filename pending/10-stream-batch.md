# One batch: gas, guards, muzzle, one re-pin

Muzzle draws, guard radii/glass and gas draws all shift the shared stream the real-damage walks are balanced on, so a re-pin on any one of them is voided by the next. Measured on master: muzzle alone 21 reds, guards alone 5, gas plus guards 23 in 30 binaries. Stop re-pinning them apart.

Branch from `origin/s/gas-guard-repin`, merge `origin/s/muzzle-on-master-2`, then `origin/s/guard-repin-master` and `origin/s/guard-repin-master-2` if it has landed, then current `origin/master`. Push to `s/stream-batch`. Keep every gas and muzzle draw.

1. Merge first and build. Keep the levers from all branches; where two touch the same walk, keep the one measured on the most recent tree and say so.
2. Re-measure the MP digests once on the joined tree: `mp_perf_16_live`, `mp_net_slots_post_move_live`. Confirm each change comes from the draws, and say how you know.
3. Run every walkreal binary whole on the joined tree and list the reds. Only then re-pin, starting with the plain walks (no real damage), then real damage, then Agent, then 00 Agent.
4. Sort each red: walk tuned to the old stream, or a behaviour change. These need a look before any lever: 0a "laser/alert channel 0x12 fired on the walked route", 03 "servant 0 left WAIT_SWITCH/HOSTAGE", 0d "the ~10 m fall is survivable", and any "walk_to ... 6000-tick cap".
5. An ignore with a measured reason is allowed for 00 Agent walks only.

Do not loosen pins tagged [L] and do not change game code to suit a walk. Guard rounds through glass are confirmed live on 0d. Finish with one total line from a full run.
