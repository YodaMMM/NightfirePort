//! gas-cel-host: the [D] census behind the type-7 wind term and the kind-0xb ceiling clamp.
//!
//! The game's `Gas_Init` 0x1aacc8 resolves its cel argument two ways: the kind-7 NULL-vel arm
//! reads `Env_GetWindVel(param_3, param_4)` 0x17c048 (the CEL's wind, cel+0x2c, animated by
//! `Env_Update` 0x17bc88), and the puff's own room link (`build_LinkToRoom` 0x1c9680, fallback
//! the caller's cel) fixes the cel whose +0x54 `gases_smoke` 0x1ab2d0 clamps kinds 0/7/0xb to.
//! This census walks every shipped level bin and records, per level [D the port's parse]:
//! - the room cels (`build_FindCel` 0x1c8cb0's containment table, `Rbodies::cels`),
//! - the cels carrying a wind radiator (`Env_Create` 0x17b968, placement kind 0xDF, param 3
//!   ·0.1 = the target — the levels where the type-7 wind term is NON-ZERO and the fix is
//!   visible; the type-7 sites themselves are code sites [M 0x1aacc8:107-116: the searchlight
//!   0x1cb900, the GT 0x180498, the mounted gun 0x1320d8, the muzzle 0x1a4428, the Effect_Bullet
//!   rows 0x1cea80] and the muzzle site fires in EVERY level),
//! - the kind-0xb puffs' reach: the flashbang's burst puff [M Bullet_DoTrails 0x12d890's
//!   `+0x70 & 0x2000` arm] and the stuck-trail big puff [M Bullet_trail_effect_smoke
//!   0x12cba8's `+0xf6 != 0` arm — the hat, 07000001].
//!
//! Opt-in: `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::rigid_body::Rbodies;

fn level_bin_names(iso: &IsoImage<std::fs::File>) -> Vec<(String, u64, u64, u64)> {
    let (entries, files) = nf_formats::disc::files_bin_entries(iso).unwrap();
    entries
        .iter()
        .filter(|e| e.name.starts_with("0700") && e.name.ends_with(".bin"))
        .map(|e| (e.name.clone(), files.lba as u64 * 2048 + e.offset as u64, e.size as u64, 0))
        .collect()
}

#[test]
fn gas_cel_wind_census_over_every_level_bin() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let bins = level_bin_names(&iso);
    assert!(bins.len() >= 20, "expected the disc's level bins, found {}", bins.len());
    let mut windy = Vec::new();
    let mut levels = 0usize;
    for (name, off, size, _) in &bins {
        let data = match iso.read_bytes(*off, *size) { Ok(d) => d, Err(_) => continue };
        let level = match Level::parse(&data) { Ok(l) => l, Err(_) => continue };
        levels += 1;
        let rbs = Rbodies::create(&level);
        // The wind radiators: cels whose 0xDF wind landed with a non-zero target (the
        // target-0 cels are skipped by Env_Update's walk [M 0x17bc88] and read as no wind
        // [M 0x17c048's Vec_Zero gate]).
        let radiators: Vec<(usize, f32)> = rbs.cels.iter().enumerate()
            .filter_map(|(ci, c)| c.wind.map(|wi| (ci, rbs.winds[wi].target)))
            .filter(|(_, t)| *t != 0.0)
            .collect();
        if !radiators.is_empty() {
            println!("{name}: {} room cels, wind radiators at cels {radiators:?}", rbs.cels.len());
            windy.push(name.clone());
        } else {
            println!("{name}: {} room cels, no wind", rbs.cels.len());
        }
        // The containment table must never panic and cel_at must agree with the build's own
        // scan for a point inside cel 0 (the lookup the gas sites resolve through).
        if let Some(c0) = rbs.cels.first() {
            let mid = [(c0.min[0] + c0.max[0]) * 0.5, (c0.min[1] + c0.max[1]) * 0.5, (c0.min[2] + c0.max[2]) * 0.5];
            assert_eq!(rbs.cel_at(mid).map(|ci| &rbs.cels[ci].min), Some(&c0.min), "{name}: cel_at misses its own cel 0");
        }
    }
    println!("levels parsed: {levels}; levels with wind radiators: {windy:?}");
    // The markers-world-objects audit's [D] census, re-proven through Rbodies: 04/05/06/08/
    // 0c/0d/14/41 carry the disc's wind radiators. (The trimmed onfoot ISO may not carry
    // every bin — assert only on the levels present.)
    for expect in ["07000004.bin", "07000005.bin", "07000006.bin", "07000008.bin"] {
        if bins.iter().any(|(n, _, _, _)| n == expect) {
            assert!(windy.iter().any(|w| w == expect), "{expect} must carry a wind radiator [D docs/levels audits]");
        }
    }
}
