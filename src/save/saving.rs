use super::SaveError;
use crate::json::JsonPatch;
use async_channel::{Receiver, Sender};
use std::hash::Hash;
use std::io::Write;
use std::path::PathBuf;

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

    Errored(SaveError),
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

    // TODO Return Result<(), SaveError> instead of `unwrap()` everything
    pub async fn save(self) {
        self.tx.send(SavingStep::LoadingArchive).await.unwrap();
        let archive = super::load_archive(&self.archive_path).await;
        let mut archive = match archive {
            Ok(a) => a,
            Err(err) => {
                self.tx.send(SavingStep::Errored(err)).await.unwrap();
                return;
            }
        };

        self.tx.send(SavingStep::ExtractingPlayer).await.unwrap();
        let (_, mut player_index) = match super::extract_player(&mut archive).await {
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
        zip.write_all(&player_bytes).unwrap();

        self.tx.send(SavingStep::FinishingArchive).await.unwrap();
        zip.finish().unwrap();

        self.tx.send(SavingStep::WritingToDisk).await.unwrap();
        // TODO

        // done, finally :)
    }
}

// TODO Rename to SavingNotifications
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
