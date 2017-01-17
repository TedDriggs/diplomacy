use std::collections::HashMap;

struct Province(String);

struct Region<'a>(&'a Province);

#[derive(Debug, Clone, Default)]
struct Builder<'a> {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region<'a>>,
}

impl<'a> Builder<'a> {
    pub fn register_province(&mut self, p: Province) {
        self.provinces.insert(p.0.clone(), p);
    }
    
    pub fn register_region(&'a mut self, r: Region<'a>) {
        self.regions.insert(r.0.0.clone(), r);
    }
    
    pub fn find_province(&'a self, s: &str) -> Option<&'a Province> {
        self.provinces.find(s)
    }
}

struct Map<'a> {
    builder: Builder<'a>
}

impl<'a> Map<'a> {
    pub fn new(b: Builder<'a>) -> Self {
        Map {
            builder: b
        }
    }
}

pub fn load<'a>() -> Map<'a> {
    let mut builder = Builder::default();
    builder.register_province(Province(String::from("Hello")));
    builder.register_region(Region(builder.find_province("Hello")));
    Map::new(builder)
}