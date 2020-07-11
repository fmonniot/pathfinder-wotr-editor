use std::path::PathBuf;
use async_trait::async_trait;
use crate::data;

// Start by declaring the public interface

#[derive(Debug, Clone)]
pub enum LoaderError {
    IoError(String),
    JsonError(String, String)
}

impl LoaderError {
    fn json_error(file_name: &str, err: serde_json::Error) -> LoaderError {
        LoaderError::JsonError(file_name.to_string(), format!("{}", err))
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(format!("{}", e))
    }
}

#[derive(Debug, Clone)]
pub enum LoaderState {
    Initialized {
        file_path: PathBuf
    },
    FileRead {
        file_path: PathBuf
    },
    /// Terminal state, user of the Loader API should stop calling `next_step` when the `current_step`
    /// is returning `PartyRead` (it will continously return the same state and won't do more computation).
    PartyRead {
        file_path: PathBuf,
        party: data::Party,
    }
}

impl LoaderState {
    pub fn completion_percentage(&self) -> u8 {
        match self {
            LoaderState::Initialized{..} => 0,
            LoaderState::FileRead{..} => 50,
            LoaderState::PartyRead{..} => 100,
        }
    }

    pub fn next_step_description(&self) -> &'static str {
        match self {
            LoaderState::Initialized{..} => "Reading file from disk",
            LoaderState::FileRead{..} => "Parsing the party information",
            LoaderState::PartyRead{..} => "All done !",
        }
    }

    pub fn file_path(&self) -> &PathBuf {
        match self {
            LoaderState::Initialized{file_path, ..} => &file_path,
            LoaderState::FileRead{file_path, ..} => &file_path,
            LoaderState::PartyRead{file_path, ..} => &file_path,
        }
    }
}

#[async_trait]
pub trait Loader {
    async fn next_step(&self) -> Result<Box<dyn Loader + Send>, LoaderError>;

    fn current_step(&self) -> LoaderState;
}

impl dyn Loader {

    pub fn new(file_path: PathBuf) -> Box<dyn Loader> {
        Box::new(Initialized { file_path })
    }
}

use core::fmt::Debug;

impl Debug for (dyn Loader + std::marker::Send + 'static) {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Loader{{{:?}}}", self)
    }
}

// And then the private implementation of the state machine

#[derive(Debug)]
struct Initialized {
    file_path: PathBuf,
}

#[async_trait]
impl Loader for Initialized {

    async fn next_step(&self) -> Result<Box<dyn Loader + Send>, LoaderError> {
        // Create archive struct with file_path
        // Verify all required files are present (header.json, party.json, etc...)
        Ok(Box::new(FileRead {
            file_path: self.file_path.clone(),
            archive: ()
        }))
    }

    fn current_step(&self) -> LoaderState {
        LoaderState::Initialized { file_path: self.file_path }
    }
}

#[derive(Debug)]
struct FileRead {
    file_path: PathBuf,
    archive: (),
}

#[async_trait]
impl Loader for FileRead {
    
    async fn next_step(&self) -> Result<Box<dyn Loader + Send>, LoaderError> {
        // TODO Should be done from the archive 
        let file_content = tokio::fs::read("samples/party.json").await?;

        let json = serde_json::from_slice(&file_content)
            .map_err(|err| LoaderError::json_error("party.json", err))?;

        let party = data::IndexedJson::new(json);

        // Get party.json and parse as JSON
        unimplemented!()
    }

    fn current_step(&self) -> LoaderState {
        LoaderState::FileRead { file_path: self.file_path }
    }
}

#[derive(Debug)]
struct PartyRead {
    file_path: PathBuf,
    archive: (),
    party: data::Party,
}

#[async_trait]
impl Loader for PartyRead {
    
    async fn next_step(&self) -> Result<Box<dyn Loader + Send>, LoaderError> {
        // TODO Should be done from the archive 
        let file_content = tokio::fs::read("samples/party.json").await?;

        let json = serde_json::from_slice(&file_content)
            .map_err(|err| LoaderError::json_error("party.json", err))?;

        let party = data::IndexedJson::new(json);

        // Get party.json and parse as JSON
        unimplemented!()
    }

    fn current_step(&self) -> LoaderState {
        LoaderState::PartyRead {
            file_path: self.file_path,
            party: self.party
        }
    }
}