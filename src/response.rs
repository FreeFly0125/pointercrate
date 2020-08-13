//! Most likely temporary location of helper functions regarding the parsing of complete server
//! responses.

use crate::{
    model::{creator::Creator, level::Level, song::NewgroundsSong},
    DeError, HasRobtopFormat,
};
use serde::export::Formatter;
use std::fmt::Display;

// Since NoneError is not stabilized, we cannot do `impl From<NoneError> for ResponseError<'_>`, so
// this is the next best thing
macro_rules! section {
    ($iter:expr) => {
        match $iter.next() {
            Some(section) => section,
            None => return Err(ResponseError::UnexpectedFormat),
        }
    };
}

#[derive(Debug)]
pub enum ResponseError<'a> {
    /// A deserializer error occured while processing some object contained in the response
    De(DeError<'a>),

    /// The response was of the form `"-1"`, which is RobTop's version of `HTTP 404 NOT FOUND`
    NotFound,

    /// The response was not worked in the expected way (too few sections, etc.)
    UnexpectedFormat,
}

impl Display for ResponseError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::De(err) => err.fmt(f),
            ResponseError::NotFound => write!(f, "not found"),
            ResponseError::UnexpectedFormat => write!(f, "unexpected format"),
        }
    }
}

impl std::error::Error for ResponseError<'_> {}

impl<'a> From<DeError<'a>> for ResponseError<'a> {
    fn from(err: DeError<'a>) -> Self {
        ResponseError::De(err)
    }
}

// TODO: Type aliases, maybe? This is pretty ridiculous lul
pub fn parse_get_gj_levels_response(response: &str) -> Result<Vec<Level<Option<NewgroundsSong>, Option<Creator>>>, ResponseError> {
    if response == "-1" {
        return Err(ResponseError::NotFound)
    }

    let mut sections = response.split('#');

    let levels = section!(sections);
    let creators = section!(sections)
        .split('|')
        .map(|fragment| Creator::from_robtop_str(fragment))
        .collect::<Result<Vec<Creator>, _>>()?;
    let songs = section!(sections)
        .split("~:~")
        .map(|fragment| NewgroundsSong::from_robtop_str(fragment))
        .collect::<Result<Vec<NewgroundsSong>, _>>()?;

    levels
        .split('|')
        .map(|fragment| {
            let level = Level::from_robtop_str(fragment)?;
            // Note: Cloning is cheap because none of the Thunks is evaluated, so we only have references lying around.
            let creator = creators.iter().find(|creator| creator.user_id == level.creator).map(Clone::clone);
            let song = level
                .custom_song
                .and_then(|song_id| songs.iter().find(|song| song.song_id == song_id))
                .map(Clone::clone);

            Ok(Level {
                level_id: level.level_id,
                name: level.name,
                description: level.description,
                version: level.version,
                creator,
                difficulty: level.difficulty,
                downloads: level.downloads,
                main_song: level.main_song,
                gd_version: level.gd_version,
                likes: level.likes,
                length: level.length,
                stars: level.stars,
                featured: level.featured,
                copy_of: level.copy_of,
                index_31: level.index_31,
                custom_song: song,
                coin_amount: level.coin_amount,
                coins_verified: level.coins_verified,
                stars_requested: level.stars_requested,
                index_40: level.index_40,
                is_epic: level.is_epic,
                index_43: level.index_43,
                object_amount: level.object_amount,
                index_46: level.index_46,
                index_47: level.index_47,
                level_data: level.level_data,
            })
        })
        .collect::<Result<_, _>>()
}

pub fn parse_download_gj_level_response(response: &str) -> Result<Level, ResponseError> {
    if response == "-1" {
        return Err(ResponseError::NotFound)
    }

    let mut sections = response.split('#');

    Ok(Level::from_robtop_str(section!(sections))?)
}
