mod header;
mod objects;
mod prelude;
mod tiles;

use crate::{
    header::{header, Header},
    objects::{objects, Objects},
    prelude::{complete::*, *},
    tiles::{tiles, Tiles},
};
pub use crate::{
    objects::{Kind, MapObjectType, Object},
    tiles::Tile,
};

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
pub struct Map<'a> {
    #[cfg_attr(feature = "serde1", serde(borrow))]
    pub header: Header<'a>,
    #[cfg_attr(feature = "serde1", serde(borrow))]
    pub tiles: Tiles<'a>,
    #[cfg_attr(feature = "serde1", serde(borrow))]
    pub objects: Objects<'a>,
}

pub fn root<'a, E: ParseError<&'a str>>(
    settings: MapParserSettings,
) -> impl Fn(&'a str) -> IResult<&'a str, Map<'a>, E> {
    move |i| {
        let (i, (header, _, tiles, _, objects, _)) = tuple((
            header,
            multispace0,
            tiles,
            multispace0,
            objects(settings.allow_any),
            multispace0,
        ))(i)?;
        let map = Map {
            header,
            tiles,
            objects,
        };
        Ok((i, map))
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    NomFormatted(String),
    Leftover(String),
}

impl Error {
    fn leftover(rest: &str) -> Self {
        Self::Leftover(rest.chars().take(120).collect())
    }
}

#[derive(Default)]
pub struct MapParserSettings {
    pub allow_any: bool,
    pub allow_tail: bool,
}

pub fn map(text: &str, settings: MapParserSettings) -> Result<Map, Error> {
    let res = root(settings)(text);
    let (rest, map) = nom_err_to_string(text, res).map_err(Error::NomFormatted)?;
    if !rest.is_empty() {
        Err(Error::leftover(rest))
    } else {
        Ok(map)
    }
}

pub fn verbose_read_file<P: AsRef<std::path::Path>, O, F>(
    path: P,
    settings: MapParserSettings,
    mut fun: F,
) -> Result<O, Error>
where
    F: for<'a> FnMut(&'a str, IResult<&'a str, Map<'a>, nom::error::VerboseError<&'a str>>) -> O,
{
    let bytes = std::fs::read(path).map_err(Error::Io)?;
    let text = std::str::from_utf8(&bytes).map_err(Error::Utf8)?;
    let res = root(settings)(text);
    Ok(fun(text, res))
}

pub fn read_map_from_file<O>(
    path: impl AsRef<std::path::Path>,
    settings: MapParserSettings,
    mut fun: impl FnMut(Map) -> O,
) -> Result<O, Error> {
    let allow_tail = settings.allow_tail;
    verbose_read_file(path, settings, |text, res| {
        let (rest, map) = nom_prelude::nom_err_to_string(text, res).map_err(Error::NomFormatted)?;
        if !allow_tail && !rest.is_empty() {
            return Err(Error::leftover(rest));
        }
        Ok(fun(map))
    })?
}

pub trait Offset {
    fn offset(&self) -> (i32, i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn show_rest(rest: &str) {
        if !rest.is_empty() {
            println!("Rest: {:?}", &rest[..rest.len().min(120)]);
        }
    }
    fn _show_objs(map: &Map) {
        for obj in &map.objects.0 {
            if obj.relations.uid.is_some()
                || obj.relations.container_uid.is_some()
                || obj.light.color.is_some()
            {
                println!("{:?}", obj)
            } else if let objects::Kind::Item {
                anim:
                    objects::Anim {
                        anim_stay_begin: Some(_),
                        ..
                    },
                ..
            } = obj.kind
            {
                println!("{:?}", obj)
            } else if let objects::Kind::Scenery {
                anim: objects::Anim {
                    offset_x: Some(_), ..
                },
                ..
            } = obj.kind
            {
                println!("{:?}", obj)
            } else if let objects::Kind::Item { val, .. } = &obj.kind {
                if val.iter().any(Option::is_some) {
                    println!("{:?}", obj)
                }
            }
        }
    }
    /*#[test]
    fn parse_testbunker() {
        verbose_read_file("../../FO4RP/maps/anuri_testbunker.fomap", |res| {
            let (rest, map) = res.unwrap();
            //println!("{:#?}", map);
            //_show_objs(&map);
            assert!(map.tiles.0.len() > 0);
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_modoc() {
        verbose_read_file("../../FO4RP/maps/modoc.fomap", |res| {
            let (rest, map) = res.unwrap();
            assert!(map.tiles.0.len() > 0);
            //_show_objs(&map);
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_phoenix() {
        verbose_read_file("../../FO4RP/maps/phoenix0.fomap", |res| {
            let (rest, map) = res.unwrap();
            assert!(map.tiles.0.len() > 0);
            //_show_objs(&map);
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_1_black() {
        verbose_read_file("../../FO4RP/maps/1_Black.fomap", |res| {
            let (rest, map) = res.unwrap();
            assert!(map.tiles.0.len() > 0);
            assert!(map.tiles.0.iter().any(|tile| tile.is_roof));
            assert!(map.tiles.0.iter().any(|tile| !tile.is_roof));
            assert!(map.tiles.0.iter().any(|tile| tile.offset.is_some()));
            assert!(map.tiles.0.iter().any(|tile| tile.layer.is_some()));
            assert!(map.objects.0.len() > 0);
            //_show_objs(&map);
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_sandbox2_3() {
        verbose_read_file("../../FO4RP/maps/sandbox2_3.fomap", |res| {
            let (rest, map) = res.unwrap();
            //tirs000
            assert!(map.objects.0.iter().any(|obj| obj.proto_id == 2007));
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_lake() {
        verbose_read_file("../../FO4RP/maps/lake.fomap", |res| {
            let (rest, _map) = res.unwrap();
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }

    #[test]
    fn parse_tanker_deck1() {
        verbose_read_file("../../FO4RP/maps/tanker_deck1.fomap", |res| {
            let (rest, _map) = res.unwrap();
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }
    #[test]
    fn parse_dreammap() {
        verbose_read_file("../../FO4RP/maps/dreammap.fomap", |_text, res| {
            let (rest, _map) = res.unwrap();
            show_rest(rest);
            assert!(rest.is_empty());
        })
        .expect("Can't read map file");
    }*/
    #[test]
    fn parse_q3_test() {
        verbose_read_file(
            "../../../FO4RP/maps/q3_test.fomap",
            Default::default(),
            |_text, res| {
                let (rest, _map) = res.unwrap();
                show_rest(rest);
                assert!(rest.is_empty());
            },
        )
        .expect("Can't read map file");
    }
    #[test]
    fn parse_all_maps() {
        for file in std::fs::read_dir("../../../FO4RP/maps/")
            .unwrap()
            .filter_map(|r| r.ok())
        {
            let file = file.path();
            if !file.is_file() || file.extension() != Some("fomap".as_ref()) {
                continue;
            }
            println!("Parsing {:?}", file);
            verbose_read_file(file, Default::default(), |text, res| {
                let (rest, _map) = nom_err_to_string(text, res).expect("Can't parse map file");
                show_rest(rest);
                assert!(rest.is_empty());
            })
            .expect("Can't read map file");
        }
    }
}
