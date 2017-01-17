use geo::{Map, Border, Coast, Terrain, MapBuilder, Province};

lazy_static! {
    static ref STANDARD_MAP : Map = load_standard();
}

pub fn standard_map() -> &'static Map {
    &STANDARD_MAP
}

fn load_standard() -> Map {
    let mut map_builder = MapBuilder::default();


    let provinces = include_str!("provinces.csv").lines().skip(1);
    for line in provinces {
        if let Ok(prov) = province_from_line(line) {
            map_builder.register_province(prov);
        }
    }


    let regions = include_str!("regions.csv").lines().skip(1);
    for line in regions {
        if let Ok((prov, coast, terrain)) = region_from_line(line) {
            map_builder.register_region(prov, coast, terrain).unwrap();
        }
    }
    
    let borders = include_str!("borders.csv").lines().skip(1);
    for line in borders {
        if let Ok(border) = border_from_line_with_context(line, &map_builder) {
            map_builder.register_border(border);
        }
    }

    Map::new(map_builder)
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

fn region_from_line<'l>(s: &'l str) -> Result<(&'l str, Option<Coast>, Terrain), ()> {
    let words = s.split(",").collect::<Vec<_>>();
    if words.len() == 3 {
        Ok((words[0], coast_from_word(words[1])?, terrain_from_word(words[2])?))
    } else {
        Err(())
    }
}

fn border_from_line_with_context(s: &str, ctx: &MapBuilder) -> Result<Border, ()> {
    let words = s.split(",").collect::<Vec<_>>();
    if words.len() == 3 {
        Ok(Border::new(ctx.find_region(words[0]).or(Err(()))?.into(),
                       ctx.find_region(words[1]).or(Err(()))?.into(),
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