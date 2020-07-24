use super::SaveError;
use crate::json::JsonPatch;
use async_channel::{Receiver, Sender};
use std::hash::Hash;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
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
    ) -> (SavingSaveGame, SubReceiver) {
        let (tx, rx) = async_channel::bounded(1);

        (
            SavingSaveGame {
                player_patches,
                party_patches,
                archive_path,
                tx,
            },
            SubReceiver(rx),
        )
    }

    // TODO Inline remaining unwrap (or if expected, use .expect instead)
    pub async fn save(self) -> Result<(), SaveError> {
        self.tx.send(SavingStep::LoadingArchive).await?;
        let mut archive = super::load_archive(&self.archive_path).await?;

        self.tx.send(SavingStep::ExtractingPlayer).await?;
        let (_, mut player_index) = super::extract_player(&mut archive).await?;

        self.tx.send(SavingStep::ExtractingParty).await?;
        let (_, mut party_index) = super::extract_party(&mut archive).await?;

        self.tx.send(SavingStep::ExtractingHeader).await?;
        let (header, mut header_index) = super::extract_header(&mut archive).await?;

        self.tx.send(SavingStep::ApplyingPatches).await?;
        for patch in &self.player_patches {
            player_index.patch(patch).unwrap();
        }
        for patch in &self.party_patches {
            party_index.patch(patch).unwrap();
        }
        header_index
            .patch(&JsonPatch::string(
                "/Name".into(),
                format!("{} Edited", header.name),
            ))
            .unwrap();

        self.tx.send(SavingStep::SerializingJson).await?;
        let player_bytes = player_index.bytes().unwrap();
        let party_bytes = party_index.bytes().unwrap();
        let header_bytes = header_index.bytes().unwrap();

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

        self.tx.send(SavingStep::WritingCustomFiles).await?;
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("player.json", options).unwrap();
        zip.write_all(&player_bytes).unwrap();

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("party.json", options).unwrap();
        zip.write_all(&party_bytes).unwrap();

        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("header.json", options).unwrap();
        zip.write_all(&header_bytes).unwrap();

        self.tx.send(SavingStep::FinishingArchive).await?;
        zip.finish().unwrap();
        drop(zip); // Release the borrow on the underlying buffer

        // TODO Will it work alright when the file already exist ?
        // Should we prompt for confirmation ?
        self.tx.send(SavingStep::WritingToDisk).await?;
        let new_file_path = copy_file_name(&self.archive_path);
        tokio::fs::write(new_file_path, write_buffer).await.unwrap();

        // done, finally :)
        Ok(())
    }
}

// TODO Rename to SaveNotifications
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

fn copy_file_name(original: &PathBuf) -> PathBuf {
    // We hardcode the file extension because it's more likely than not
    // it won't be anything else. We could use `.extension()` if it turns
    // out this assertion is wrong.
    let new_file_name = original
        .file_stem()
        .map(|name| format!("{} - Copy.zks", name.to_string_lossy()))
        .unwrap_or_else(|| "Unknown Save Name.zks".to_string());

    original.with_file_name(new_file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_file_name_example() {
        let path = "/tmp/Save Game.zks".into();
        let actual = copy_file_name(&path);
        let expected: PathBuf = "/tmp/Save Game - Copy.zks".into();

        println!("{:?}: {:?}", path.extension(), path.file_name());

        assert_eq!(actual, expected);
    }
}
