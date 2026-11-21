# Driving string tables (`.loc`, magic `LOCH`)

Reader: `crates/nf-formats/src/loc.rs`. Disc test: `crates/nf-formats/tests/sfn_loc_disc.rs` (golden
`golden/loc_census.tsv` from `tools/crosscheck/sfn_loc_census.py`).

Evidence tags: [M] DRIVING.ELF decompile/disassembly, [D] disc data.

## Files [D]

`data\gfx\locale\IDL_{BRT,DUT,ENG,FR,GER,ITA,NA,SPA,SWE}.loc` in MISC.VIV, plain. Each holds 1,530
ids and 1,530 strings. `IDL_NA.loc` has short placeholder strings and no DRIVING.ELF string names it.

## Which file [M]

`FUN_0024afe0` → `FUN_0024b068(FUN_00249f98())` → `FUN_0024b138("data/gfx/locale/%s")`.
`FUN_00249f98` maps the hand-over language word 0x100988: 0..5 → 0..5, 8 → 6, else 7; the files by
case: 0 BRT, 1 FR, 2 GER, 3 SPA, 4 ITA, 5 DUT, 6 SWE, 7 ENG. (`IDL_ENG.LOC` at 0x374e38 is a separate
string main stores at 0x100548.)

## Layout [M `FUN_00259360`]

```
LOCH  +0x04 u32 id-table offset  +0x08 flags (bit 0 = ids go through the id table)
      +0x0c u16 table count (1)  +0x0e u16 table in use (0)  +0x10 u32[count] table offsets
LOCI  +0x08 u32 n  +0x10 n × { u16 id, u16 index }, sorted by id
LOCL  +0x0c u32 n  +0x10 n × u32 offset from the LOCL start → NUL-terminated bytes
```

Lookup: `id & 0xffff` is binary-searched (`FUN_002592e0`, u16 compare `FUN_002592c0`); absent → −1;
a negative or out-of-range index → NULL. So ids like 99999 resolve through 0x869f. `FUN_0024b1a0`
returns `"MISSING"` (0x3ac600) for NULL; with the debug word DAT_0033ba70 set (only by the toggle at
0x24b210) it prints the id with `"%d"`.

Strings are bytes for the font (0x80-0xff are glyphs in loading.sfn); 3 strings per language hold a
`%`, which matters because the text draw formats the string as a printf format (sfn.md). Button
prompts use the tokens `*A*`, `*G*`, `*P*`, `*D*`; `*X*` also appears in 8 strings but is not a token
the prompt pass knows (docs/systems/driving-hud.md).
