use crate::data::{self, IndexedJson, Party, Player};
use std::path::PathBuf;

// Start by declaring the public interface

#[derive(Debug, Clone, PartialEq)]
pub enum LoaderError {
    IoError(String),
    SerdeError(String, String),
    JsonError(String, String),
}

impl LoaderError {
    fn serde_error(file_name: &str, err: serde_json::Error) -> LoaderError {
        LoaderError::SerdeError(file_name.to_string(), format!("{}", err))
    }

    fn json_error(file_name: &str, err: data::JsonReaderError) -> LoaderError {
        LoaderError::JsonError(file_name.to_string(), format!("{:?}", err))
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(format!("{}", e))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStep {
    Initialized,
    ReadingFile,
    ReadingParty,
    ReadingPlayer,
    Done { party: Party, player: Player },
    Error(LoaderError),
}

impl LoadingStep {
    pub fn completion_percentage(&self) -> u8 {
        match self {
            LoadingStep::Initialized => 0,
            LoadingStep::ReadingFile => 33,
            LoadingStep::ReadingParty => 55,
            LoadingStep::ReadingPlayer => 77,
            LoadingStep::Done { .. } => 100,
            LoadingStep::Error(..) => 0,
        }
    }

    pub fn next_description(&self) -> String {
        match self {
            LoadingStep::Initialized => "Initialized".to_string(),
            LoadingStep::ReadingFile => "Reading file from disk".to_string(),
            LoadingStep::ReadingParty => "Parsing the party information".to_string(),
            LoadingStep::ReadingPlayer => "Parsing the player information".to_string(),
            LoadingStep::Done { .. } => "All done !".to_string(),
            LoadingStep::Error(error) => format!("Error: {:?}", error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Loader {
    file_path: PathBuf,
}

impl Loader {
    pub fn new(file_path: PathBuf) -> Loader {
        Loader { file_path }
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for Loader
where
    H: std::hash::Hasher,
{
    type Output = LoadingStep;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.file_path.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            Initializing {
                file_path: self.file_path,
            },
            |state| async move { state.exec().await },
        ))
    }
}

#[allow(dead_code)] // Until we got the archive extraction stuff in place
enum LSM {
    Initializing { file_path: PathBuf },
    ReadingFile { file_path: PathBuf },
    ReadingParty { archive: () },
    ReadingPlayer { archive: (), party: Party },
    // This state is reached after completion, whether it's because of an error or a normal end.
    Terminated,
}

use LSM::*;

impl LSM {
    async fn exec(self) -> Option<(LoadingStep, LSM)> {
        match self {
            Initializing { file_path } => {
                Some((LoadingStep::ReadingFile, ReadingFile { file_path }))
            }
            ReadingFile { .. } => {
                // Create archive struct with file_path
                // Verify all required files are present (header.json, party.json, etc...)

                Some((LoadingStep::ReadingParty, ReadingParty { archive: () }))
            }
            ReadingParty { archive } => {
                // TODO Use the archive instead of reading from disk
                match extract_party(&archive).await {
                    Ok(party) => {
                        Some((LoadingStep::ReadingPlayer, ReadingPlayer { archive, party }))
                    }
                    Err(error) => Some((LoadingStep::Error(error), Terminated)),
                }
            }
            ReadingPlayer { archive, party } => {
                // TODO Use the archive instead of reading from disk
                match extract_player(&archive).await {
                    Ok(player) => Some((LoadingStep::Done { party, player }, Terminated)),
                    Err(error) => Some((LoadingStep::Error(error), Terminated)),
                }
            }
            Terminated => None,
        }
    }
}

async fn extract_party(_archive: &()) -> Result<Party, LoaderError> {
    // TODO Should be done from the archive
    let file_content = tokio::fs::read("samples/party.json").await?;

    let json = serde_json::from_slice(&file_content)
        .map_err(|err| LoaderError::serde_error("party.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let party =
        data::read_party(indexed_json).map_err(|err| LoaderError::json_error("party.json", err))?;

    Ok(party)
}

async fn extract_player(_archive: &()) -> Result<Player, LoaderError> {
    // TODO Should be done from the archive
    let file_content = tokio::fs::read("samples/player.json").await?;

    let json = serde_json::from_slice(&file_content)
        .map_err(|err| LoaderError::serde_error("player.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let player = data::read_player(indexed_json)
        .map_err(|err| LoaderError::json_error("player.json", err))?;

    Ok(player)
}
