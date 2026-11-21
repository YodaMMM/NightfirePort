# s/drive-audio: the driving missions' sound data and sound logic

No shared random stream is touched: new modules only. No `nf-view` change, so nothing plays yet.

## What landed

- `nf-formats/src/easnd.rs` covers the PT header reader and patch builder (DRIVING.ELF `FUN_00312300`,
  `FUN_00312270`, `FUN_0030b318`) and the `BNKl` banks (`FUN_00308088`, both memory-class blocks).
  It has the EA-XA decoder from the IOP driver's own `decxa16c` (SNDDRV.IRX keeps its symbols) and
  the `.asf` streams (`SCHl`/`SCCl`/`SCDl`/`SCEl`, `FUN_00308d68`'s channel layout). It also reads
  `banks.ini`, the `.h` id headers (including the game's key normalisation and case-insensitive
  lookup, `FUN_002f54e0`) and the mix `.ini` (`FUN_002f3058`). RefPack banks go through the
  existing `refpack.rs`.
- `nf-game/src/driving/audio.rs` adds the startup choices: the speech archive per track and
  language with the English fallback, the menu volumes, and the bank slots (`FUN_00176658`,
  `FUN_002eb7a8`). It adds the mix-group tree with its effective volume and fade step
  (`FUN_002f34d0`), the voice gate (`FUN_002ddfd0`), and the engine layers (`FUN_002fa8f0` +
  `FUN_002ff320`). The engine gives rpm/load slews, pitch = (rpm+2000)/4500, the low/high crossfade,
  the interior/exterior blend, and the vacuum one-shot with its pitch wobble (`FUN_002cbd50` is a
  sine). The vehicle base step (`FUN_002e1768`) gives the ±150 rpm wobble, three scrapes, the
  front/rear skids and the per-surface skid voices. Also there: speed smoothing (`FUN_002df7c0`),
  player road noise (`FUN_002f0900`), the sniper bolt one-shot (`FUN_00124650`) and the stream
  channel kinds (`FUN_0015d000`).
- Docs: `docs/formats/driving-audio.md`, `docs/systems/drive-audio.md`.

## Measured on the disc [D]

- 56 banks, 1,379 filled slots and 1,468 patches (1,219 VAG, 249 EA-XA) all decode to exactly
  their sample count (30,331,875 samples). Each patch's data lies inside its memory class's block,
  and the two blocks tile every bank file. Every `.h` id names a filled slot.
- 236 streams (118 stereo, all EA-XA at 48 kHz, 80,326 data blocks) walk to `SCEl`. Each one's
  block-sample sum equals its header count, and all decode (495,747,423 samples over channels).
- Found from the data: EA-XA bank sounds carry a 16-byte header (codec byte, loop points, count,
  equal to the PT values in all 249). Each stream channel block starts with 4 bytes before the
  frames: without the skip, 18,881 frame bytes name an impossible coefficient; with it, none.
- On the engine and vehicle voices: the player engine keys resolve only on the Vanquish tracks
  (MIS01, MIS4, RACE), and `vanquish.h` has no `midload` keys, so two layers are silent. The
  sniper bolt resolves only on MIS01. Linked mix groups missing from a track's ini are silent
  (volume 0); the per-track list is pinned.

## Tests (all green, NF_SILENT=1)

- `nf-formats --lib` (123, incl. 7 `easnd` unit tests); `nf-formats --test drive_audio_disc`
  (2, goldens `golden/driving_audio_{banks,streams}.tsv`; with the 4-byte/16-byte skips removed
  both fail).
- `nf-game --lib driving::audio` (10 unit tests pinning the traced constants and formulas);
  `nf-game --test drive_audio_logic_disc` (1, golden `crates/nf-game/tests/drive_audio_logic.tsv`).
- No walkreal binary covers driving; nf-view is untouched.

## Unproven / not done

- What the IOP does with a stream block's 4 leading bytes. They read as two LE i16 close to the
  decoder history. The port carries history across blocks: the joins are smoother than zeroing it
  (133M vs 867M summed |step|) and about equal to loading the 4 bytes (137M).
- The IOP's output clamp/mix: the port saturates each sample to i16.
- Which mission-script event starts which music or speech stream. The stream names are `CHAR::`
  strings in the track `.crp`, and those event records are not decoded.
- Not traced: the scrape counter writer, horn timing, collision impact sounds, 3-D panning,
  traffic/AI and non-Vanquish vehicle sound classes, other weapon sounds, and the patch-block tag
  meanings.
- The listener values (gain, pitch, camera blend) and the vehicle's +0x6c volume are inputs here;
  their producers are not traced.
