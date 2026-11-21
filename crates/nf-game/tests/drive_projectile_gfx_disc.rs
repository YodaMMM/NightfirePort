//! Disc test for the driving projectile graphics (`nf_formats::driveproj`,
//! `nf_game::driving::projectile_gfx`, docs/systems/driving-projectile-gfx.md): on every driving
//! archive the lighting tuning's projectile/explosion lights, the ribbon and streak textures, and
//! every car model's weapon items with the projectile model each barrel copies.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::BTreeMap;

use nf_formats::driveproj::{action_items, ProjectileLighting, BlastTune, GlowTune, EXPLOSION_EFFECTS, STREAK_TEXTURES, TRAIL_TEXTURE};
use nf_formats::gal::Gal;
use nf_formats::iso9660::IsoImage;
use nf_formats::ssh::Ssh;
use nf_formats::trackfx::giotto_anim;
use nf_game::driving::projectile_gfx::barrel_models;
use nf_game::driving::weapons::{GUN, MISSILE, Q_BOOSTER, Q_PULSE, Q_SMOKE, REMOTE_TORPEDO, ROCKET, TORPEDO, WEAPON_COUNT};

const TRACKS: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn viv_files(iso: &IsoImage, viv: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut out = Vec::new();
    for x in &g.entries {
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
        if raw.len() >= 64 && raw.iter().take(64).all(|&b| b == 0) {
            return None;
        }
        let d = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap_or(raw) } else { raw };
        out.push((x.name.to_ascii_lowercase().replace('\\', "/"), d));
    }
    Some(out)
}

const C1: [f32; 4] = [0.04, 0.03, 0.02, 1.0];

/// (track, Missile size, SnowmobileRocket colour, SnowmobileRocket size) [D].
const GLOW: &[(&str, f32, [f32; 4], f32)] = &[
    ("MIS01", 1.9206, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("MIS11", 1.9206, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("MIS13A", 1.9206, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("MIS13B", 1.9803, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("MIS13C", 1.9206, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("MIS3", 1.9206, [0.22, 0.42, 0.48, 1.0], 0.3087),
    ("MIS4", 2.0201, [0.0, 0.0, 0.1, 1.0], 5.0),
    ("RACE", 2.0201, [0.0, 0.0, 0.1, 1.0], 5.0),
];

#[test]
fn every_track_tunes_the_projectile_and_explosion_lights() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    for &(viv, missile, snow_c, snow_s) in GLOW {
        let Some(files) = viv_files(&iso, viv) else { return };
        let tun = &files.iter().find(|(n, _)| n.starts_with("data/tuning/render/lighting/")).unwrap().1;
        let t = ProjectileLighting::parse(&String::from_utf8_lossy(tun));
        assert_eq!(t.glow[0], GlowTune { colour: [0.4, 0.4, 0.26, 1.0], size: missile }, "{viv} Missile");
        assert_eq!(t.glow[1], GlowTune { colour: [0.1, 0.1, 0.05, 1.0], size: 5.0 }, "{viv} Rocket");
        assert_eq!(t.glow[2], GlowTune { colour: snow_c, size: snow_s }, "{viv} SnowmobileRocket");
        assert_eq!(t.glow[3], GlowTune { colour: [0.0; 4], size: 0.0 }, "{viv} EMPInflight");
        assert_eq!(t.glow[4], GlowTune { colour: [0.0; 4], size: 0.0 }, "{viv} EMPTarget");
        let want = [(1.0, 35.0), (0.8, 90.0), (0.4, 120.0), (0.2, 180.0), (0.1, 240.0)];
        for (i, &(size, decay)) in want.iter().enumerate() {
            assert_eq!(t.blast[i], BlastTune { colour: C1, size, decay }, "{viv} blast {i}");
        }
    }
}

#[test]
fn common_sheet_holds_the_ribbon_and_streak_textures_and_the_gallery_names() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else { return };
        let ssh = Ssh::parse(&files.iter().find(|(n, _)| n == "data/render/common.ssh").unwrap().1).unwrap();
        let tex = |n: &str| ssh.textures.iter().find(|t| t.name == n).map(|t| (t.width, t.height));
        assert_eq!(tex(TRAIL_TEXTURE), Some((64, 64)), "{viv} trai");
        for s in STREAK_TEXTURES {
            assert_eq!(tex(s), Some((32, 16)), "{viv} {s}");
        }
    }
    let Some(misc) = viv_files(&iso, "MISC") else { return };
    let mut report = Vec::new();
    for g in ["gall1.gal", "sav_gall1.gal", "ears_gall1.gal"] {
        let gal = Gal::parse(&misc.iter().find(|(n, _)| n.ends_with(g) && n.contains("gfx")).unwrap().1).unwrap();
        let missing: Vec<&str> = EXPLOSION_EFFECTS.iter().chain(["Wp_Rock", "Wp_EMP", "Wp_EMPtail", "Dummy"].iter()).copied().filter(|n| giotto_anim(&gal, n).is_none()).collect();
        eprintln!("{g}: missing {missing:?}");
        report.push((g, missing));
    }
    // [D] measured: every gallery holds Wp_Rock; the jungle gallery has no Q-Pulse effects;
    // FODHitEffect, rocketsplash and Dummy are in none of them, so those effects do not appear
    // ("Could not locate Giotto effect", FUN_00232e18).
    let got: Vec<(&str, Vec<&str>)> = report.iter().map(|(g, m)| (*g, m.clone())).collect();
    assert_eq!(got, vec![
        ("gall1.gal", vec!["FODHitEffect", "snowspecificexplosion", "rocketsplash", "Dummy"]),
        ("sav_gall1.gal", vec!["FODHitEffect", "rocketsplash", "Un_ExplL", "Un_ExplM", "Un_ExplS", "Dummy"]),
        ("ears_gall1.gal", vec!["snowspecificexplosion", "Un_ExplL", "Un_ExplM", "Un_ExplS", "Wp_EMP", "Wp_EMPtail", "Dummy"]),
    ]);
}

#[test]
fn every_car_model_names_its_projectile_models() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    // car name → weapon → (action, item, model item, launch effect)
    let mut cars: BTreeMap<String, BTreeMap<usize, Vec<(u8, usize, usize, Option<String>)>>> = BTreeMap::new();
    let mut files_seen = 0;
    for &viv in TRACKS.iter().chain(["MISC"].iter()) {
        let Some(files) = viv_files(&iso, viv) else { return };
        for (n, d) in &files {
            if !(n.starts_with("data/car/model/") && n.ends_with(".crp")) {
                continue;
            }
            files_seen += 1;
            let items = action_items(d).unwrap_or_else(|e| panic!("{viv} {n}: {e}"));
            let car = n.trim_start_matches("data/car/model/").trim_end_matches(".crp").to_owned();
            let e = cars.entry(car).or_default();
            for w in 0..WEAPON_COUNT {
                let m: Vec<_> = barrel_models(&items, w).into_iter().map(|b| (b.action, b.item, b.model_item, b.launch_fx)).collect();
                if !m.is_empty() {
                    let prev = e.insert(w, m.clone());
                    assert!(prev.is_none() || prev == Some(m), "{viv} {n}: differs from another archive's copy");
                }
            }
        }
    }
    for (car, ws) in &cars {
        for (w, m) in ws {
            eprintln!("{car} w{w}: {m:?}");
        }
    }
    assert!(files_seen > 50, "{files_seen} car files");
    let get = |c: &str, w: usize| cars.get(c).and_then(|m| m.get(&w)).cloned().unwrap_or_default();
    let rock = Some("GIOT::Wp_Rock".to_owned());
    // vanquish: both rocket pods copy item 39 (the mirrored item 40 shares its effect list).
    assert_eq!(get("vanquish", ROCKET), vec![(5, 39, 39, rock.clone()), (6, 40, 39, rock.clone())]);
    assert_eq!(get("vanquish", MISSILE), vec![(7, 47, 47, rock.clone()), (8, 48, 47, rock.clone()), (0x34, 30, 30, rock.clone()), (0x35, 31, 30, rock.clone())]);
    assert_eq!(get("vanquish", Q_PULSE), vec![(0x24, 14, 14, Some("GIOT::Wp_EMPtail".into())), (0x24, 14, 14, Some("GIOT::Wp_EMP".into()))]);
    assert_eq!(get("vanquish", GUN), vec![(9, 36, 36, Some("GIOT::Dummy".into()))]);
    assert_eq!(get("vanquish", Q_SMOKE), vec![(0xe, 15, 15, Some("GIOT::Cr_Smoke".into()))]);
    assert_eq!(get("vanquish", Q_BOOSTER).len(), 2);
    let torp = Some("GIOT::Cr_Torp".to_owned());
    assert_eq!(get("vanquishsub", TORPEDO), vec![(0x3d, 22, 22, torp.clone()), (0x3e, 23, 22, torp.clone())]);
    assert_eq!(get("vanquishsub", REMOTE_TORPEDO), vec![(0x3f, 12, 12, torp.clone()), (0x40, 13, 12, torp.clone())]);
    assert_eq!(get("alpha_sub", TORPEDO), vec![(0x3d, 2, 2, Some("GIOT::Cr_Torpbig".into())), (0x3e, 3, 2, Some("GIOT::Cr_Torpbig".into()))]);
    assert_eq!(get("ultralight", ROCKET), vec![(5, 16, 16, rock.clone()), (6, 15, 15, rock.clone())]);
    assert_eq!(get("supersnow", ROCKET), vec![(5, 17, 17, rock.clone()), (6, 16, 16, rock.clone())]);
    assert_eq!(get("jungle_truck", MISSILE), vec![(7, 18, 18, rock.clone()), (0x34, 35, 35, rock.clone())]);
    assert_eq!(get("subcopter", MISSILE), vec![(7, 6, 6, rock.clone()), (8, 7, 6, rock.clone())]);
    // the cars with any barrel model on the disc
    assert_eq!(cars.iter().filter(|(_, w)| !w.is_empty()).map(|(c, _)| c.as_str()).collect::<Vec<_>>(), ["alpha_sub", "jungle_hench", "jungle_truck", "jungleheli", "junglesnow", "mini_sub", "openheli", "small_snowmobile", "subcopter", "supersnow", "ultralight", "ultralightbig", "vanquish", "vanquishsub"]);
}
