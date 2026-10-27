# WORKLOG

## level-4a-ride-bond-float — started

Branch `s/level-4a-ride-bond-float` from origin/master (6ff39ea9). Task: Bond floats above the
deck in the 0700004a lift-ride landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53,
-3.38)); find his stream + sequence roots [D] and the game's placement rule [M], port it scoped
so 07000014 stays unchanged. Predecessor: level-4a-ride-cast-b (de77afd4) decoded the
anim-object placement and pinned the car-local roots; its live checks were the compose-site
discriminator. Note: origin/master no longer carries docs/WORKLOG.md (stripped by
batch-1025g/4c1f5ffb process-file cleanup); this file is recreated fresh with this entry.

Note for the decode: origin/s/cutscene-kind31-frame-2 (5f55744d, not yet on master) landed a
closely-related rule — an unparented case-1 stream concats si+0x00, which holds the SP
placement base from Script_Load [M 0x1d8248:148] until Script_CameraStart overwrites it with
the identity, never restored [M 0x1d916c, 0x1db348]. Reading that diff before touching nis.rs.

## level-4a-ride-bond-float — the entity compose rule resolved to the metal

Session 2 (continuation): the general lifecycle rule landed in nis.rs interp case 1 —
`keys_alone = camera_started && parent < 0`. Binary-proven this session on ACTION.ELF:

- si+0x00 = the SP placement at load [M 0x1d8248 View_RotTransMatrix(param_3,param_2,si):148-149].
- `Script_CameraStart` 0x1d9140 calls `Mat_IdentityT(si)` UNCONDITIONALLY before its flag-0x20
  check — a0 untouched from entry to the jal at 0x1d916c [M disasm].
- Never restored: `Script_CameraEnd` 0x1db348 only kills the stream and clears +0xab5 bit 0x10.
- Case 1, unparented: `Script_GetInterp`'s 10th arg = `(stream.parent == -1)` — `lb v1,37(s1)`,
  `nor`+`sltiu`, `sw v1,0(sp)` at 0x1d9a7c-0x1d9a98 [M disasm] — and GetInterp's tail is
  `if (param_10) Concat(si, out)` [M 0x1da158:195-197]. So an unparented stream composes si+0x00
  EVERY tick, with the camera-op lifecycle on its VALUE.
- Case 1, parented: the chain-walk GetInterp calls pass 0 [M 0x1d9b30 `sw zero,0(sp)`] and
  Script_Interp concats si once at the chain top [M 0x1d9b9c] — so parented streams ride the same
  lifecycle via the chain top (the port's per-stream `mul(&m, &self.matrix)` for parented streams
  is NOT yet camera-gated — scoped follow-up, no affected scene pinned).
- Case 2 (anims) composes nothing [M 0x1d9a08:140-199 re-read in full] — the cast draws on its
  raw root track; the pack 0x060006cf's cast roots (0.85/1.15) stand on its raw-keyed floor
  (0.99→-5.33) [D]. The float = the port composing the SP base into the pack's post-camera
  entities (floor → -52.5..-58.9) under a keys-alone cast.

Probes (opt-in, NF_ISO): ride_pack_raw_probe / ride_pack_keys_probe / ride_cab_keys_probe
(nf-formats), ride_cast_roots_probe (nf-view). ride_pack_streams_probe dropped (stale API).
