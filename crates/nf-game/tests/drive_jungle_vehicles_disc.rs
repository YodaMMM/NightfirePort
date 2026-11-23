//! Disc test: every pvehicle `.atr` of the nine `/DRIVING/` archives through the port's two
//! readers (`sub_physics::VehicleAttrs::parse_atr`, `physics::CarParams::parse_atr`), the model
//! its `RENDER_FILENAME` names, and the pass `FUN_00188200` gives it (`sub_physics::drive_pass`)
//! as the player (driver 0) and as a script car (driver 2) [M 0x188b70-0x188c60]. The spawn
//! vehicles per mission are the `MSet` names of docs/systems/drive-player-vehicles.md [D].
//! Opt-in via NF_ISO (a slim image without the `/DRIVING/` extents skips).

use nf_formats::iso9660::IsoImage;
use nf_game::driving::physics::CarParams;
use nf_game::driving::sub_physics::{drive_pass, DrivePass, VehicleAttrs};

const ARCHIVES: [&str; 9] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE", "MISC"];

struct Atr {
    viv: &'static str,
    stem: String,
    text: String,
    /// `RENDER_FILENAME` (with or without the `.s` suffix) and whether that `data\car\model\`
    /// file is in the same archive.
    render: Option<(String, bool)>,
}

fn atrs() -> Option<Vec<Atr>> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut out = Vec::new();
    for viv in ARCHIVES {
        let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
        let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
        for x in &g.entries {
            let n = x.name.to_ascii_lowercase();
            let Some(stem) = n.strip_prefix(r"data\sim\attrib\pvehicle\").and_then(|s| s.strip_suffix(".atr")) else { continue };
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
            if raw.iter().take(16).all(|&b| b == 0) {
                eprintln!("skip: slim image");
                return None;
            }
            let b = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            let text = String::from_utf8_lossy(&b).into_owned();
            let render = text.lines().find_map(|l| {
                let (k, v) = l.split("//").next()?.split_once('=')?;
                (k.trim().split('.').next()? == "RENDER_FILENAME").then(|| v.trim().to_ascii_lowercase())
            });
            let render = render.map(|r| {
                let here = g.entries.iter().any(|y| y.name.eq_ignore_ascii_case(&format!(r"data\car\model\{r}")));
                (r, here)
            });
            out.push(Atr { viv, stem: stem.to_owned(), text, render });
        }
    }
    Some(out)
}

#[test]
fn every_pvehicle_atr_decodes_names_a_model_beside_it_and_gets_its_pass() {
    let Some(atrs) = atrs() else { return };
    let mut table = Vec::new();
    for a in &atrs {
        let va = VehicleAttrs::parse_atr(&a.text);
        let cp = CarParams::parse_atr(&a.text);
        // MASS reaches both readers alike.
        assert_eq!(va.mass, cp.mass, "{} {}", a.viv, a.stem);
        // The two boolean-valued keys the readers would drop (`.b=true`) are none of theirs:
        // every flag the pass reads is written as a number on the disc [D].
        for l in a.text.lines() {
            let l = l.split("//").next().unwrap_or("").trim();
            if let Some((k, v)) = l.split_once('=') {
                let v = v.trim();
                if v == "true" || v == "false" {
                    let k = k.trim().split('.').next().unwrap();
                    assert!(["IS_HELICOPTER", "NO_SHADOW", "TYRES_INVUNERABLE"].contains(&k), "{} {}: {l}", a.viv, a.stem);
                }
            }
        }
        if a.stem == "default" {
            assert!(a.render.is_none(), "{}", a.viv);
            continue;
        }
        let (render, here) = a.render.clone().unwrap_or_else(|| panic!("{} {}: no RENDER_FILENAME", a.viv, a.stem));
        assert!(here, "{} {}: {render} not in the archive", a.viv, a.stem);
        let pass = |d: u32| match drive_pass(&va, d, false) {
            DrivePass::Sub => "sub",
            DrivePass::Snowmobile => "snow",
            DrivePass::PlayerLand => "land",
            DrivePass::AiLand => "ai-land",
            DrivePass::Path => "path",
        };
        table.push(format!("{} {} {} {}/{}", a.viv, a.stem, render.trim_end_matches(".crp"), pass(0), pass(2)));
    }
    let want = [
        "MIS01 paris_henchbigbox paris_henchbigbox land/ai-land",
        "MIS01 paris_bombvan paris_bombvan land/ai-land",
        "MIS01 paradis_car paradis_car land/ai-land",
        "MIS01 courier_truck courier_truck land/ai-land",
        "MIS01 road_block_truck road_block_truck land/ai-land",
        "MIS01 helicopter helicopter land/ai-land",
        "MIS01 vanquish vanquish land/ai-land",
        "MIS01 hatchback hatchback land/ai-land",
        "MIS01 sedan2 sedan2 land/ai-land",
        "MIS01 suv1 suv1 land/ai-land",
        "MIS11 alpha_sub alpha_sub sub/sub",
        "MIS11 mini_sub mini_sub sub/sub",
        "MIS11 helimine helimine land/ai-land",
        "MIS11 vanquishsub vanquishsub sub/sub",
        "MIS13A jungle_hench jungle_hench land/ai-land",
        "MIS13A ultralight ultralight sub/sub",
        "MIS13A jungleheli jungleheli land/ai-land",
        "MIS13A staticultralight staticultralight land/ai-land",
        "MIS13A jungle_truck jungle_truck land/ai-land",
        "MIS13B jungle_hench jungle_hench land/ai-land",
        "MIS13B jungle_bombvan paris_bombvan land/ai-land",
        "MIS13B ultralight ultralight sub/sub",
        "MIS13B jungletank jungletank land/ai-land",
        "MIS13B patrolboat patrolboat land/ai-land",
        "MIS13C ultralightbig ultralightbig sub/sub",
        "MIS13C patrolboat patrolboat land/ai-land",
        "MIS13C jungletank jungletank land/ai-land",
        "MIS13C fodbase fodbase land/ai-land",
        "MIS3 small_snowmobile small_snowmobile snow/snow",
        "MIS3 supersnow supersnow snow/snow",
        "MIS3 openheli openheli land/ai-land",
        "MIS3 junglesnow junglesnow land/ai-land",
        "MIS3 paris_hench paris_hench land/ai-land",
        "MIS3 paris_bombvan paris_bombvan land/ai-land",
        "MIS4 alpine_police alpine_police land/ai-land",
        "MIS4 supersnow supersnow snow/snow",
        "MIS4 subcopter subcopter sub/sub",
        "MIS4 paris_hench paris_hench land/ai-land",
        "MIS4 plane plane sub/sub",
        "MIS4 vanquishalps vanquish land/ai-land",
        "MIS4 coupe coupe land/ai-land",
        "MIS4 sedan2 sedan2 land/ai-land",
        "MIS4 hatchback hatchback land/ai-land",
        "MIS4 sedan sedan land/ai-land",
        "MIS4 station_wagon station_wagon land/ai-land",
        "MIS4 suv1 suv1 land/ai-land",
        "MIS4 taxi taxi land/ai-land",
        "RACE paris_hench paris_hench land/ai-land",
        "RACE alpine_police alpine_police land/ai-land",
        "RACE supersnow supersnow snow/snow",
        "RACE cobra_player paradis_car land/ai-land",
        "RACE coupe coupe land/ai-land",
        "RACE sedan2 sedan2 land/ai-land",
        "RACE hatchback hatchback land/ai-land",
        "RACE sedan sedan land/ai-land",
        "RACE station_wagon station_wagon land/ai-land",
        "RACE suv1 suv1 land/ai-land",
        "RACE taxi taxi land/ai-land",
        "MISC junglesnow junglesnow land/ai-land",
        "MISC jungle_truck jungle_truck land/ai-land",
    ];
    assert_eq!(table, want);
}

#[test]
fn the_three_land_spawn_vehicles_read_their_tuning() {
    let Some(atrs) = atrs() else { return };
    let get = |viv: &str, stem: &str| atrs.iter().find(|a| a.viv == viv && a.stem == stem).unwrap_or_else(|| panic!("{viv} {stem}"));
    // (archive, spawn vehicle, MASS, WHEEL_RADIUS, SPRING_REST_LENGTH, SPRING_COMPRESSION_LIMIT,
    // MAXACC, MAXSTEERING, IS_RALLY) [D the .atr lines].
    let rows = [
        ("MIS01", "helicopter", 1660.0, 0.315, 0.35, 0.75, 2.65, 0.077, false),
        ("MIS13A", "jungle_truck", 1900.0, 0.425, 0.32, 0.75, 2.8, 0.0825, true),
        ("MISC", "jungle_truck", 1900.0, 0.425, 0.32, 0.75, 2.8, 0.0825, true),
        ("MIS13C", "fodbase", 3200.0, 0.575, 0.31, 0.75, 13.29, 0.1295, false),
    ];
    for (viv, stem, mass, wr, rest, limit, acc, steer, rally) in rows {
        let a = get(viv, stem);
        let cp = CarParams::parse_atr(&a.text);
        assert_eq!((cp.mass, cp.wheel_radius, cp.spring_rest, cp.spring_limit, cp.maxacc, cp.maxsteering, cp.is_rally), (mass, wr, rest, limit, acc, steer, rally), "{viv} {stem}");
        let va = VehicleAttrs::parse_atr(&a.text);
        assert_eq!((va.sub_physics, va.is_snowmobile, va.is_boat, va.is_sub), (0, 0, 0, 0), "{viv} {stem}");
        // Off a spline the player land pass; on one the path pass; a spline-driven car keeps
        // driver index 1 (EControlToSpline), which off the spline would be the AI land pass.
        assert_eq!(drive_pass(&va, 0, false), DrivePass::PlayerLand);
        assert_eq!(drive_pass(&va, 1, true), DrivePass::Path);
        assert_eq!(drive_pass(&va, 1, false), DrivePass::AiLand);
    }
    // MIS13A's jungle_truck.atr is byte-identical to MISC's [D].
    assert_eq!(get("MIS13A", "jungle_truck").text, get("MISC", "jungle_truck").text);
    // The helicopter's only flight keys are IS_HELICOPTER / NO_SHADOW (booleans no pass reads)
    // and SECONDARY_TYPE=vanquish; the pass sees a land car [M FUN_00188200].
    let h = &get("MIS01", "helicopter").text;
    assert!(h.contains("IS_HELICOPTER.b=true") && h.contains("SECONDARY_TYPE.s=vanquish"));
}
