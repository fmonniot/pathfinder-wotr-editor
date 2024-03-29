use crate::data::{self, Header, Party, Player};
use crate::json::{IndexedJson, JsonError};
use async_channel::SendError;
use std::path::Path;

mod loading;
mod saving;

pub use loading::{LoadNotifications, LoadingDone, LoadingStep, SaveLoader};
pub use saving::{SaveNotifications, SavingSaveGame, SavingStep};
// use save::{SaveLoader, SaveError, LoadingStep};

#[derive(Debug, Clone, PartialEq)]
pub enum SaveError {
    Io(String),
    Serde(String, String),
    Json(String, String),
    Zip(String),
    SavingNotifications(SendError<SavingStep>),
    LoadingNotifications(SendError<LoadingStep>),
}

impl SaveError {
    fn serde_error(file_name: &str, err: serde_json::Error) -> SaveError {
        SaveError::Serde(file_name.to_string(), format!("{}", err))
    }

    fn json_error(file_name: &str, err: JsonError) -> SaveError {
        SaveError::Json(file_name.to_string(), format!("{:?}", err))
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(format!("{}", e))
    }
}

impl From<zip::result::ZipError> for SaveError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(format!("{}", e))
    }
}

impl From<SendError<SavingStep>> for SaveError {
    fn from(e: SendError<SavingStep>) -> Self {
        Self::SavingNotifications(e)
    }
}

impl From<SendError<LoadingStep>> for SaveError {
    fn from(e: SendError<LoadingStep>) -> Self {
        Self::LoadingNotifications(e)
    }
}

// Function commons to loading and saving

type InMemoryArchive = zip::ZipArchive<std::io::Cursor<std::vec::Vec<u8>>>;

async fn extract_party(archive: &mut InMemoryArchive) -> Result<(Party, IndexedJson), SaveError> {
    let file = archive.by_name("party.json")?;

    let json =
        serde_json::from_reader(file).map_err(|err| SaveError::serde_error("party.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let party =
        data::read_party(&indexed_json).map_err(|err| SaveError::json_error("party.json", err))?;

    Ok((party, indexed_json))
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

async fn extract_header(archive: &mut InMemoryArchive) -> Result<(Header, IndexedJson), SaveError> {
    let file = archive.by_name("header.json")?;

    let json =
        serde_json::from_reader(file).map_err(|err| SaveError::serde_error("header.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let header = data::read_header(&indexed_json)
        .map_err(|err| SaveError::json_error("header.json", err))?;

    Ok((header, indexed_json))
}

async fn load_archive(path: &Path) -> Result<InMemoryArchive, SaveError> {
    let buf = tokio::fs::read(path).await?;
    let reader = std::io::Cursor::new(buf);
    let archive = zip::ZipArchive::new(reader)?;

    // verify archive contains required files ahead of time
    let exists = |s: &str| {
        archive
            .file_names()
            .find(|n| n == &s)
            .ok_or_else(|| SaveError::Zip(format!("{} file not found in archive", s)))
    };

    exists("header.json")?;
    exists("party.json")?;
    exists("player.json")?;

    Ok(archive)
}
