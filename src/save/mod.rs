use crate::data::{self, Party, Player};
use crate::json::{IndexedJson, JsonReaderError};
use std::path::PathBuf;

mod loading;
mod saving;

pub use loading::{LoadingStep, SaveLoader};
pub use saving::{SavingSaveGame, SavingStep, SubReceiver};
// use save::{SaveLoader, SaveError, LoadingStep};

#[derive(Debug, Clone, PartialEq)]
pub enum SaveError {
    IoError(String),
    SerdeError(String, String),
    JsonError(String, String),
    ZipError(String),
}

impl SaveError {
    fn serde_error(file_name: &str, err: serde_json::Error) -> SaveError {
        SaveError::SerdeError(file_name.to_string(), format!("{}", err))
    }

    fn json_error(file_name: &str, err: JsonReaderError) -> SaveError {
        SaveError::JsonError(file_name.to_string(), format!("{:?}", err))
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(format!("{}", e))
    }
}

impl From<zip::result::ZipError> for SaveError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::ZipError(format!("{}", e))
    }
}

// Function commons to loading and saving

type InMemoryArchive = zip::ZipArchive<std::io::Cursor<std::vec::Vec<u8>>>;

async fn extract_party(archive: &mut InMemoryArchive) -> Result<Party, SaveError> {
    let file = archive.by_name("party.json")?;

    let json =
        serde_json::from_reader(file).map_err(|err| SaveError::serde_error("party.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let party =
        data::read_party(&indexed_json).map_err(|err| SaveError::json_error("party.json", err))?;

    Ok(party)
}

async fn extract_player(archive: &mut InMemoryArchive) -> Result<(Player, IndexedJson), SaveError> {
    let file = archive.by_name("player.json")?;

    let json =
        serde_json::from_reader(file).map_err(|err| SaveError::serde_error("player.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let player = data::read_player(&indexed_json)
        .map_err(|err| SaveError::json_error("player.json", err))?;

    Ok((player, indexed_json))
}

async fn load_archive(path: &PathBuf) -> Result<InMemoryArchive, SaveError> {
    let buf = tokio::fs::read(path).await?;
    let reader = std::io::Cursor::new(buf);
    let archive = zip::ZipArchive::new(reader)?;

    contains_required_file(&archive)?;

    Ok(archive)
}

// TODO Check presente of player.json, party.json and header.json
fn contains_required_file(archive: &InMemoryArchive) -> Result<(), SaveError> {
    let exists = |s: &str| {
        archive
            .file_names()
            .find(|n| n == &s)
            .ok_or_else(|| SaveError::ZipError(format!("{} file not found in archive", s)))
    };

    exists("header.json")?;
    exists("party.json")?;
    exists("player.json")?;

    Ok(())
}