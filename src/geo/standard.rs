use crate::geo::builder::ProvinceRegistry;
use crate::geo::{Coast, Map, Province, SupplyCenter, Terrain};
use lazy_static::lazy_static;

lazy_static! {
    static ref STANDARD_MAP: Map = load_standard();
}

/// Gets a static reference to the standard game world map.
/// See [this SVG](https://upload.wikimedia.org/wikipedia/commons/a/a3/Diplomacy.svg)
/// for the source names and borders.
pub fn standard_map() -> &'static Map {
    &STANDARD_MAP
}

fn load_standard() -> Map {
    let mut prov_reg = ProvinceRegistry::default();
    let provinces = include_str!("provinces.csv").lines().skip(1);
    for line in provinces {
        if let Ok(prov) = province_from_line(line) {
            prov_reg
                .register(prov)
                .expect("standard map shouldn't have issues");
        } else {
            panic!(format!("Failed registering province: {}", line))
        }
    }

    let mut region_reg = prov_reg.finish();
    let regions = include_str!("regions.csv").lines().skip(1);
    for line in regions {
        if let Ok((prov, coast, terrain)) = region_from_line(line) {
            region_reg.register(prov, coast, terrain).unwrap();
        } else {
            panic!(format!("Failed registering region: {}", line))
        }
    }

    let mut border_reg = region_reg.finish();
    let borders = include_str!("borders.csv").lines().skip(1);
    for line in borders {
        let words = line.split(',').collect::<Vec<_>>();
        border_reg
            .register(words[0], words[1], terrain_from_word(words[2]).unwrap())
            .unwrap();
    }

    border_reg.finish()
}

fn province_from_line(s: &str) -> Result<Province, ()> {
    let words = s.split(',').collect::<Vec<_>>();
    if words.len() == 3 {
        Ok(Province {
            short_name: String::from(words[0]),
            full_name: String::from(words[1]),
            supply_center: supply_center_from_word(words[2]),
        })
    } else {
        Err(())
    }
}

fn supply_center_from_word(s: &str) -> SupplyCenter {
    match s {
        "" => SupplyCenter::None,
        "neutral" => SupplyCenter::Neutral,
        nat => SupplyCenter::Home(nat.into()),
    }
}

fn region_from_line(s: &str) -> Result<(&str, Option<Coast>, Terrain), ()> {
    let words = s.split(',').collect::<Vec<_>>();
    if words.len() == 3 {
        Ok((
            words[0],
            coast_from_word(words[1])?,
            terrain_from_word(words[2])?,
        ))
    } else {
        Err(())
    }
}

fn coast_from_word(w: &str) -> Result<Option<Coast>, ()> {
    match w {
        "" => Ok(None),
        "n" => Ok(Some(Coast::North)),
        "e" => Ok(Some(Coast::East)),
        "s" => Ok(Some(Coast::South)),
        "w" => Ok(Some(Coast::West)),
        _ => Err(()),
    }
}

fn terrain_from_word(w: &str) -> Result<Terrain, ()> {
    match w {
        "sea" => Ok(Terrain::Sea),
        "coast" => Ok(Terrain::Coast),
        "land" => Ok(Terrain::Land),
        _ => Err(()),
    }
}
