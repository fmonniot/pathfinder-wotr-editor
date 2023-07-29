use super::SaveError;
use crate::data::Header;
use crate::json::JsonPatch;
use async_channel::{Receiver, Sender};
use iced::advanced::{
    subscription::{EventStream, Recipe},
    Hasher,
};
use std::hash::Hash;
use std::io::Write;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Copy)]
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
}

impl SavingStep {
    /// The progress of the step in the overall save process
    /// within the [steps_range()].
    pub fn number(&self) -> f32 {
        match self {
            SavingStep::LoadingArchive => 0.0,
            SavingStep::ExtractingPlayer => 1.0,
            SavingStep::ExtractingParty => 2.0,
            SavingStep::ExtractingHeader => 3.0,
            SavingStep::ApplyingPatches => 4.0,
            SavingStep::SerializingJson => 5.0,
            SavingStep::WritingArchive => 6.0,
            SavingStep::WritingCustomFiles => 7.0,
            SavingStep::FinishingArchive => 8.0,
            SavingStep::WritingToDisk => 9.0,
        }
    }

    /// Range of steps to use with progress bar. The last step in the range
    /// is not reachable as it's designed with the progress bar disappearing
    /// when the last step is done.
    pub fn steps_range() -> RangeInclusive<f32> {
        0.0..=10.0
    }
}

pub struct SavingSaveGame {
    player_patches: Vec<JsonPatch>,
    party_patches: Vec<JsonPatch>,
    archive_path: PathBuf,
    tx: Sender<SavingStep>,
}

impl SavingSaveGame {
    pub fn new(
        player_patches: Vec<JsonPatch>,
        party_patches: Vec<JsonPatch>,
        archive_path: PathBuf,
    ) -> (SavingSaveGame, SaveNotifications) {
        let (tx, rx) = async_channel::bounded(1);

        (
            SavingSaveGame {
                player_patches,
                party_patches,
                archive_path,
                tx,
            },
            SaveNotifications(rx),
        )
    }

    pub async fn save(self) -> Result<(), SaveError> {
        self.tx.send(SavingStep::LoadingArchive).await?;
        let mut archive = super::load_archive(&self.archive_path).await?;

        self.tx.send(SavingStep::ExtractingPlayer).await?;
        let (_, mut player_index) = super::extract_player(&mut archive).await?;

        self.tx.send(SavingStep::ExtractingParty).await?;
        let (_, mut party_index) = super::extract_party(&mut archive).await?;

        self.tx.send(SavingStep::ExtractingHeader).await?;
        let (header, mut header_index) = super::extract_header(&mut archive).await?;
        let (new_save_name, new_file_path) =
            find_appropriate_save_name(&self.archive_path, &header).await?;

        self.tx.send(SavingStep::ApplyingPatches).await?;
        for patch in &self.player_patches {
            player_index
                .patch(patch)
                .map_err(|err| SaveError::json_error("player.json", err))?;
        }
        for patch in &self.party_patches {
            party_index
                .patch(patch)
                .map_err(|err| SaveError::json_error("party.json", err))?;
        }
        header_index
            .patch(&JsonPatch::string("/Name".into(), new_save_name))
            .map_err(|err| SaveError::json_error("header.json", err))?;

        self.tx.send(SavingStep::SerializingJson).await?;
        let player_bytes = player_index
            .bytes()
            .expect("player's JSON couldn't be serialized");
        let party_bytes = party_index
            .bytes()
            .expect("party's JSON couldn't be serialized");
        let header_bytes = header_index
            .bytes()
            .expect("header's JSON couldn't be serialized");

        let not_modified_files: Vec<_> = archive
            .file_names()
            .filter(|n| n != &"header.json" && n != &"party.json" && n != &"player.json")
            .map(str::to_string)
            .collect();

        let mut write_buffer: Vec<u8> = vec![];
        let w = std::io::Cursor::new(&mut write_buffer);
        let mut zip = zip::ZipWriter::new(w);

        self.tx.send(SavingStep::WritingArchive).await?;
        for file in not_modified_files {
            let mut original = archive
                .by_name(&file)
                .expect("Archive contained file by not really oO");
            let options = zip::write::FileOptions::default()
                .compression_method(original.compression())
                .last_modified_time(original.last_modified());
            let options = match original.unix_mode() {
                Some(m) => options.unix_permissions(m),
                None => options,
            };

            zip.start_file(original.name(), options)
                .expect("Starting file");
            std::io::copy(&mut original, &mut zip).expect("Copying original file to new archive");
        }

        self.tx.send(SavingStep::WritingCustomFiles).await?;
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("player.json", options)
            .expect("Starting file player.json");
        zip.write_all(&player_bytes)
            .expect("Writing player_bytes to archive");

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("party.json", options)
            .expect("Starting file party.json");
        zip.write_all(&party_bytes)
            .expect("Writing party_bytes to archive");

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("header.json", options)
            .expect("Starting file header.json");
        zip.write_all(&header_bytes)
            .expect("Writing header_bytes to archive");

        self.tx.send(SavingStep::FinishingArchive).await?;
        zip.finish().expect("Finishing zip archive");
        drop(zip); // Release the borrow on the underlying buffer

        self.tx.send(SavingStep::WritingToDisk).await?;
        tokio::fs::write(new_file_path, write_buffer).await?;

        // done, finally :)
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SaveNotifications(Receiver<SavingStep>);

impl Recipe for SaveNotifications {
    type Output = SavingStep;

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

async fn find_appropriate_save_name(
    archive_path: &Path,
    header: &Header,
) -> Result<(String, PathBuf), SaveError> {
    // Let's iterate on copy number starting at one
    // 1 is a special case where we don't include the number
    // (i)   is define file name
    // (ii)  check if file exists
    //        if exists -> go to next number
    //        if not -> next step
    // (iii) Return not existing path and associated save game name
    let base_file_name = archive_path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Save Name".to_string());
    let mut iteration = 1;

    while iteration < 10 {
        // define path
        let file_name = if iteration == 1 {
            format!("{}__Copy.zks", base_file_name)
        } else {
            format!("{}__Copy{}.zks", base_file_name, iteration)
        };
        let path = archive_path.with_file_name(file_name);

        if tokio::fs::metadata(&path).await.is_ok() {
            // file exists, try next one
            iteration += 1;
            continue;
        } else {
            // file don't exist, let's use it

            let save_name = if iteration == 1 {
                format!("{} - Edited", header.name)
            } else {
                format!("{} - Edited {}", header.name, iteration)
            };

            return Ok((save_name, path));
        }
    }

    Err(SaveError::Io(
        "Can't find a good file name in 10 iterations".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempdir::TempDir;

    #[tokio::test]
    async fn find_save_name_no_file() {
        let tmp_dir = TempDir::new("find_save_name_no_files").unwrap();
        let file_path = tmp_dir.path().join("Save Game.zks");
        let header = Header {
            name: "Save Name".to_string(),
            compatibility_version: 1,
        };

        let (a, b) = find_appropriate_save_name(&file_path, &header)
            .await
            .unwrap();

        assert_eq!(a, "Save Name - Edited".to_string());
        assert_eq!(b, tmp_dir.path().join("Save Game__Copy.zks"));
    }

    #[tokio::test]
    async fn find_save_name_one_file() {
        let tmp_dir = TempDir::new("find_save_name_one_file").unwrap();
        let file_path = tmp_dir.path().join("Save Game__Copy.zks");
        let mut tmp_file = File::create(&file_path).unwrap();
        writeln!(tmp_file, "content doesn't matter").unwrap();

        let header = Header {
            name: "Save Name".to_string(),
            compatibility_version: 1,
        };

        let (a, b) = find_appropriate_save_name(&tmp_dir.path().join("Save Game.zks"), &header)
            .await
            .unwrap();

        assert_eq!(a, "Save Name - Edited 2".to_string());
        assert_eq!(b, tmp_dir.path().join("Save Game__Copy2.zks"));
    }
}
