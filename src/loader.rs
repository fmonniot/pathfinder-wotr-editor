use std::path::PathBuf;
use crate::data;

// Start by declaring the public interface

#[derive(Debug, Clone, PartialEq)]
pub enum LoaderError {
    IoError(String),
    SerdeError(String, String),
    JsonError(String, String)
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
    Done {
        party: data::Party
    },
    Error(LoaderError)
}

impl LoadingStep {
    pub fn completion_percentage(&self) -> u8 {
        match self {
            LoadingStep::Initialized => 0,
            LoadingStep::ReadingFile => 33,
            LoadingStep::ReadingParty => 33,
            LoadingStep::Done{..} => 100,
            LoadingStep::Error(..) => 0
        }
    }

    pub fn next_description(&self) -> String {
        match self {
            LoadingStep::Initialized => "Initialized".to_string(),
            LoadingStep::ReadingFile => "Reading file from disk".to_string(),
            LoadingStep::ReadingParty => "Parsing the party information".to_string(),
            LoadingStep::Done{..} => "All done !".to_string(),
            LoadingStep::Error(error) => format!("Error: {:?}", error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Loader {
    file_path: PathBuf
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
        Box::pin(futures::stream::unfold(Initializing { file_path: self.file_path }, |state| async move {
            state.exec().await
        }))
    }
}

enum LSM {
    Initializing { file_path: PathBuf },
    ReadingFile { file_path: PathBuf },
    ReadingParty { file_path: PathBuf, archive: () },
    // This state is reached after completion, whether it's because of an error or a normal end.
    Terminated
}

use LSM::*;

impl LSM {

    async fn exec(self) -> Option<(LoadingStep, LSM)> {
        match self {
            Initializing { file_path } => Some((LoadingStep::ReadingFile, ReadingFile { file_path } )),
            ReadingFile { file_path } => {
                // Create archive struct with file_path
                // Verify all required files are present (header.json, party.json, etc...)

                Some((LoadingStep::ReadingParty, ReadingParty { file_path, archive: ()}))
            },
            ReadingParty { archive, .. } => {

                // TODO Use the archive instead of reading from disk
                // self.current_step = LoadingStep::ReadingParty;
                match extract_party(&archive).await {
                    Ok(party) => Some((LoadingStep::Done { party }, Terminated)),
                    Err(error) => Some((LoadingStep::Error(error), Terminated))
                }
            },
            Terminated => None,
        }
    }

}

async fn extract_party(_archive: &()) -> Result<data::Party, LoaderError> {
    // TODO Should be done from the archive 
    let file_content = tokio::fs::read("samples/party.json").await?;

    let json = serde_json::from_slice(&file_content)
        .map_err(|err| LoaderError::serde_error("party.json", err))?;

    let indexed_json = data::IndexedJson::new(json);

    let party = data::read_party(indexed_json)
        .map_err(|err| LoaderError::json_error("party.json", err))?;

    Ok(party)
}