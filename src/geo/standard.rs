use std::collections::HashMap;
use geo::{Map, Region, Border, Coast, Terrain, MapBuilder, Province};
use ShortName;

pub fn standard_map() -> &'static Map<'static> {
    unimplemented!()
}

fn load_standard<'a>() -> Map<'a> {
    let mut map_builder = MapBuilder::default();
    
    let mut prov_map = HashMap::new();
    let provinces = include_str!("provinces.csv").lines().skip(1);
    for line in provinces {
        if let Ok(prov) = province_from_line(line) {
            prov_map.insert(prov.short_name(), prov);
        }
    }
    
    let mut reg_map = HashMap::new();
    let regions = include_str!("regions.csv").lines().skip(1);
    for line in regions {
        if let Ok(reg) = region_from_line_with_context(line, &prov_map) {
            reg_map.insert(reg.short_name(), reg);
        }
    }
    
    let (p, r, b) = map_builder.dump_contents();
    Map::new(prov_map, reg_map, b)
}

fn province_from_line(s: &str) -> Result<Province, ()> {
    let words = s.split(",").collect::<Vec<_>>();
    if words.len() == 2 {
        Ok(Province {
            short_name: String::from(words[0]),
            full_name: String::from(words[1]),
        })
    } else {
        Err(())
    }
}

fn region_from_line_with_context<'a>(s: &str, ctx: &'a HashMap<String, Province>) -> Result<Region<'a>, ()> {
    let words = s.split(",").collect::<Vec<_>>();
    if words.len() == 3 {
        Ok(Region::new(ctx.get(words[0]).ok_or(())?,
                       coast_from_word(words[1])?,
                       terrain_from_word(words[2])?))
    } else {
        Err(())
    }
}

fn border_from_line_with_context<'a>(s: &str, ctx: &'a HashMap<String, Region<'a>>) -> Result<Border<'a>, ()> {
    let words = s.split(",").collect::<Vec<_>>();
    if words.len() == 3 {
        Ok(Border::new(ctx.get(words[0]).ok_or(())?,
                       ctx.get(words[1]).ok_or(())?,
                       terrain_from_word(words[2])?))
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