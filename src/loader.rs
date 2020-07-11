use std::path::PathBuf;
use crate::data;

// Start by declaring the public interface

pub fn init(file_path: PathBuf) -> LoaderState {
    LoaderState::Initialized(LoaderStateMachine::new(file_path))
}

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


// TODO Can I rename this to Loader and have a dedicated LoaderState which remove all the internal machinery
// and can be exposed to the application ?
// Maybe something like LoadingStep ?
// This enum is used to let Rust know the exact size to allocate. We could also try it out with a trait
// which would expose two methods: async compute_next_step and current_step.
// This might mean we can't use the stack and will have to allocate on the heap.
// Let's go with a dyn trait + Box for the storage in main, and a LoadingStep enum for state representation
#[derive(Debug, Clone)]
pub enum LoaderState {
    Initialized(LoaderStateMachine<Initialized>),
    FileRead(LoaderStateMachine<FileRead>),
    PartyRead(LoaderStateMachine<PartyRead>)
}

impl LoaderState {

    // We may not be able to consume the loader here
    // Or if we do, we need a way to represent the current state independently and store
    // that in the main application state.
    pub async fn next_step(self) -> Result<LoaderState, LoaderError> {
        match self {
            LoaderState::Initialized(l) =>  Ok(LoaderState::FileRead(l.next().await?)),
            LoaderState::FileRead(l) => Ok(LoaderState::PartyRead(l.next().await?)),
            LoaderState::PartyRead(p) => Ok(LoaderState::PartyRead(p)),
        }
    }

    pub fn completion_percentage(&self) -> u8 {
        match self {
            LoaderState::Initialized(..) => 0,
            LoaderState::FileRead(..) => 50,
            LoaderState::PartyRead(..) => 100,
        }
    }

    pub fn next_step_description(&self) -> &'static str {
        match self {
            LoaderState::Initialized(..) => "Reading file from disk",
            LoaderState::FileRead(..) => "Parsing the party information",
            LoaderState::PartyRead(..) => "All done !",
        }
    }

    pub fn file_path(&self) -> &PathBuf {
        match self {
            LoaderState::Initialized(l) => &l.file_path,
            LoaderState::FileRead(l) => &l.file_path,
            LoaderState::PartyRead(l) => &l.file_path,
        }
    }

}

// And then the private implementation of the state machine

#[derive(Debug, Clone)]
struct LoaderStateMachine<S> {
    step: S,
    file_path: PathBuf,
}

#[derive(Debug, Clone)]
struct Initialized;

impl LoaderStateMachine<Initialized> {
    fn new(file_path: PathBuf) -> LoaderStateMachine<Initialized> {
        LoaderStateMachine {
            step: Initialized,
            file_path
        }
    }

    async fn next(self) -> Result<LoaderStateMachine<FileRead>, LoaderError> {
        // Create archive struct with file_path
        // Verify all required files are present (header.json, party.json, etc...)
        Ok(LoaderStateMachine {
            file_path: self.file_path,
            step: FileRead {
                archive: ()
            }
        })
    }
}

#[derive(Debug, Clone)]
struct FileRead {
    archive: ()
}

impl LoaderStateMachine<FileRead> {

    async fn next(self) -> Result<LoaderStateMachine<PartyRead>, LoaderError> {
        // TODO Should be done from the archive 
        let file_content = tokio::fs::read("samples/party.json").await?;

        let json = serde_json::from_slice(&file_content)
            .map_err(|err| LoaderError::json_error("party.json", err))?;

        let party = data::IndexedJson::new(json);

        // Get party.json and parse as JSON
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
struct PartyRead {
    party: data::Party
}