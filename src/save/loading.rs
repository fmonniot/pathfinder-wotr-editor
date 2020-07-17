use super::{InMemoryArchive, SaveError};
use crate::data::{Party, Player};
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStep {
    Initialized,
    ReadingFile,
    ReadingParty,
    ReadingPlayer,
    Done(Box<LoadingDone>),
    Error(SaveError),
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
pub struct SaveLoader {
    file_path: PathBuf,
}

impl SaveLoader {
    pub fn new(file_path: PathBuf) -> SaveLoader {
        SaveLoader { file_path }
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for SaveLoader
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
                let res = super::load_archive(&file_path).await;

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
            } => match super::extract_party(&mut archive).await {
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
            } => match super::extract_player(&mut archive).await {
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
