use crate::data::{self, Party, Player};
use crate::json::{IndexedJson, JsonPatch, JsonReaderError};
use async_channel::{Receiver, Sender};
use std::hash::Hash;
use std::io::Write;
use std::path::PathBuf;

// Start by declaring the public interface

#[derive(Debug, Clone, PartialEq)]
pub enum LoaderError {
    IoError(String),
    SerdeError(String, String),
    JsonError(String, String),
    ZipError(String),
}

impl LoaderError {
    fn serde_error(file_name: &str, err: serde_json::Error) -> LoaderError {
        LoaderError::SerdeError(file_name.to_string(), format!("{}", err))
    }

    fn json_error(file_name: &str, err: JsonReaderError) -> LoaderError {
        LoaderError::JsonError(file_name.to_string(), format!("{:?}", err))
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(format!("{}", e))
    }
}

impl From<zip::result::ZipError> for LoaderError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::ZipError(format!("{}", e))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStep {
    Initialized,
    ReadingFile,
    ReadingParty,
    ReadingPlayer,
    Done(Box<LoadingDone>),
    Error(LoaderError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingDone {
    pub party: Party,
    pub player: Player,
    pub archive_path: PathBuf,
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

#[derive(Clone, Debug)]
pub enum SavingStep {
    LoadingArchive,
    ExtractingPlayer,
    ExtractingParty,
    ExtractingHeader,
    ApplyingPatches,
    SerializingJson,
    WritingArchive,
    WritingCustomFiles,
    FinishingArchive,
    WritingToDisk,

    Errored(LoaderError),
}

pub struct SavingSaveGame {
    patches: Vec<JsonPatch>,
    archive_path: PathBuf,
    tx: Sender<SavingStep>,
}

impl SavingSaveGame {
    pub fn new(patches: Vec<JsonPatch>, archive_path: PathBuf) -> (SavingSaveGame, SubReceiver) {
        let (tx, rx) = async_channel::bounded(1);

        (
            SavingSaveGame {
                patches,
                archive_path,
                tx,
            },
            SubReceiver(rx),
        )
    }

    // TODO Return Result<(), LoaderError> instead of `unwrap()` everything
    pub async fn save(self) {
        self.tx.send(SavingStep::LoadingArchive).await.unwrap();
        let archive = load_archive(&self.archive_path).await;
        let mut archive = match archive {
            Ok(a) => a,
            Err(err) => {
                self.tx.send(SavingStep::Errored(err)).await.unwrap();
                return;
            }
        };

        self.tx.send(SavingStep::ExtractingPlayer).await.unwrap();
        let (_, mut player_index) = match extract_player(&mut archive).await {
            Ok(p) => p,
            Err(err) => {
                self.tx.send(SavingStep::Errored(err)).await.unwrap();
                return;
            }
        };

        self.tx.send(SavingStep::ExtractingParty).await.unwrap();
        // TODO

        self.tx.send(SavingStep::ExtractingHeader).await.unwrap();
        // TODO

        self.tx.send(SavingStep::ApplyingPatches).await.unwrap();
        for patch in &self.patches {
            player_index.patch(patch).unwrap();
        }

        self.tx.send(SavingStep::SerializingJson).await.unwrap();
        let player_bytes = player_index.bytes().unwrap();
        // TODO party & header

        let not_modified_files: Vec<_> = archive
            .file_names()
            .filter(|n| n != &"header.json" && n != &"party.json" && n != &"player.json")
            .map(str::to_string)
            .collect();

        let buf: &mut Vec<u8> = &mut vec![];
        let w = std::io::Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(w);

        self.tx.send(SavingStep::WritingArchive).await.unwrap();
        for file in not_modified_files {
            let mut original = archive.by_name(&file).unwrap();
            let options = zip::write::FileOptions::default()
                .compression_method(original.compression())
                .last_modified_time(original.last_modified());
            let options = match original.unix_mode() {
                Some(m) => options.unix_permissions(m),
                None => options,
            };

            zip.start_file(original.name(), options).unwrap();
            std::io::copy(&mut original, &mut zip).unwrap();
        }

        self.tx.send(SavingStep::WritingCustomFiles).await.unwrap();
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("player.json", options).unwrap();
        zip.write(&player_bytes).unwrap();

        self.tx.send(SavingStep::FinishingArchive).await.unwrap();
        zip.finish().unwrap();

        self.tx.send(SavingStep::WritingToDisk).await.unwrap();
        // TODO

        // done, finally :)
    }
}

#[derive(Clone, Debug)]
pub struct SubReceiver(Receiver<SavingStep>);

impl<H, I> iced_native::subscription::Recipe<H, I> for SubReceiver
where
    H: std::hash::Hasher,
{
    type Output = SavingStep;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(self.0)
    }
}

type InMemoryArchive = zip::ZipArchive<std::io::Cursor<std::vec::Vec<u8>>>;

// TODO Rewrite using the pattern above (channel for notification + async fn driving the read)
enum LSM {
    Initializing {
        file_path: PathBuf,
    },
    ReadingFile {
        file_path: PathBuf,
    },
    ReadingParty {
        file_path: PathBuf,
        archive: InMemoryArchive,
    },
    ReadingPlayer {
        file_path: PathBuf,
        archive: InMemoryArchive,
        party: Party,
    },
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
            ReadingFile { file_path } => {
                let res = load_archive(&file_path)
                    .await
                    .and_then(|archive| contains_required_file(&archive).map(|_| archive));

                match res {
                    Ok(archive) => Some((
                        LoadingStep::ReadingParty,
                        ReadingParty { archive, file_path },
                    )),
                    Err(error) => Some((LoadingStep::Error(error), Terminated)),
                }
            }
            ReadingParty {
                mut archive,
                file_path,
            } => match extract_party(&mut archive).await {
                Ok(party) => Some((
                    LoadingStep::ReadingPlayer,
                    ReadingPlayer {
                        archive,
                        party,
                        file_path,
                    },
                )),
                Err(error) => Some((LoadingStep::Error(error), Terminated)),
            },
            ReadingPlayer {
                mut archive,
                party,
                file_path,
            } => match extract_player(&mut archive).await {
                Ok((player, _)) => Some((
                    LoadingStep::Done(Box::new(LoadingDone {
                        party,
                        player,
                        archive_path: file_path,
                    })),
                    Terminated,
                )),
                Err(error) => Some((LoadingStep::Error(error), Terminated)),
            },
            Terminated => None,
        }
    }
}

async fn extract_party(archive: &mut InMemoryArchive) -> Result<Party, LoaderError> {
    let file = archive.by_name("party.json")?;

    let json =
        serde_json::from_reader(file).map_err(|err| LoaderError::serde_error("party.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let party = data::read_party(&indexed_json)
        .map_err(|err| LoaderError::json_error("party.json", err))?;

    Ok(party)
}

async fn extract_player(
    archive: &mut InMemoryArchive,
) -> Result<(Player, IndexedJson), LoaderError> {
    let file = archive.by_name("player.json")?;

    let json = serde_json::from_reader(file)
        .map_err(|err| LoaderError::serde_error("player.json", err))?;

    let indexed_json = IndexedJson::new(json);

    let player = data::read_player(&indexed_json)
        .map_err(|err| LoaderError::json_error("player.json", err))?;

    Ok((player, indexed_json))
}

async fn load_archive(path: &PathBuf) -> Result<InMemoryArchive, LoaderError> {
    let buf = tokio::fs::read(path).await?;

    let reader = std::io::Cursor::new(buf);

    Ok(zip::ZipArchive::new(reader)?)
}

// TODO Check presente of player.json, party.json and header.json
fn contains_required_file(archive: &InMemoryArchive) -> Result<(), LoaderError> {
    let exists = |s: &str| {
        archive
            .file_names()
            .find(|n| n == &s)
            .ok_or_else(|| LoaderError::ZipError(format!("{} file not found in archive", s)))
    };

    exists("header.json")?;
    exists("party.json")?;
    exists("player.json")?;

    Ok(())
}
