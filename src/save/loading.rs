use super::SaveError;
use crate::data::{Header, Party, Player};
use async_channel::{Receiver, Sender};
use iced::advanced::subscription::{EventStream, Hasher, Recipe};
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStep {
    Initialized,
    ReadingFile,
    ReadingParty,
    ReadingPlayer,
    ReadingHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingDone {
    pub header: Header,
    pub party: Party,
    pub player: Player,
    pub archive_path: PathBuf,
}

impl LoadingStep {
    pub fn step_number(&self) -> f32 {
        match self {
            LoadingStep::Initialized => 0.0,
            LoadingStep::ReadingFile => 1.0,
            LoadingStep::ReadingParty => 2.0,
            LoadingStep::ReadingPlayer => 3.0,
            LoadingStep::ReadingHeader => 5.0,
        }
    }

    pub fn total_steps() -> f32 {
        4.0
    }

    pub fn description(&self) -> String {
        match self {
            LoadingStep::Initialized => "Initialized".to_string(),
            LoadingStep::ReadingFile => "Reading file from disk".to_string(),
            LoadingStep::ReadingParty => "Parsing the party information".to_string(),
            LoadingStep::ReadingPlayer => "Parsing the player information".to_string(),
            LoadingStep::ReadingHeader => "Parsing the save information".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveLoader {
    file_path: PathBuf,
    tx: Sender<LoadingStep>,
}

impl SaveLoader {
    pub fn new(file_path: PathBuf) -> (SaveLoader, LoadNotifications) {
        let (tx, rx) = async_channel::bounded(1);

        (SaveLoader { file_path, tx }, LoadNotifications(rx))
    }

    pub async fn load(self) -> Result<LoadingDone, SaveError> {
        self.tx.send(LoadingStep::ReadingFile).await?;
        let mut archive = super::load_archive(&self.file_path).await?;

        self.tx.send(LoadingStep::ReadingParty).await?;
        let (party, _) = super::extract_party(&mut archive).await?;

        self.tx.send(LoadingStep::ReadingPlayer).await?;
        let (player, _) = super::extract_player(&mut archive).await?;

        self.tx.send(LoadingStep::ReadingHeader).await?;
        let (header, _) = super::extract_header(&mut archive).await?;

        Ok(LoadingDone {
            party,
            player,
            header,
            archive_path: self.file_path,
        })
    }
}

#[derive(Clone, Debug)]
pub struct LoadNotifications(Receiver<LoadingStep>);

// Make sure iced can use our download stream
impl Recipe for LoadNotifications {
    type Output = LoadingStep;

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: EventStream,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(self.0)
    }
}
