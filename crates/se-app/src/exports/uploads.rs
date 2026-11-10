//! The renderer's side of a running export: progress, uploads and its outcome, passed to the
//! export's job through its inbox.

use se_domain::ExportId;
use tokio::sync::oneshot;

use super::{ExportMessage, ExportService};
use crate::{
    AppError,
    ports::{ExportStream, MediaError},
};

impl ExportService {
    /// The renderer's progress: `done` of `total` units.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::ExportNotRendering`].
    pub async fn report_progress(
        &self,
        export_id: ExportId,
        done: u64,
        total: u64,
    ) -> Result<(), AppError> {
        self.send(export_id, ExportMessage::Progress { done, total })
            .await
    }

    /// The renderer uploaded everything: the core completes the file.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::ExportNotRendering`].
    pub async fn finish_render(&self, export_id: ExportId) -> Result<(), AppError> {
        self.send(export_id, ExportMessage::Finished).await
    }

    /// The renderer could not render: the export fails with `message`.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::ExportNotRendering`].
    pub async fn fail_render(&self, export_id: ExportId, message: String) -> Result<(), AppError> {
        self.send(export_id, ExportMessage::Failed { message })
            .await
    }

    /// Writes uploaded bytes of `stream` at `position`, once the job wrote them.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::ExportNotRendering`], [`AppError::Media`] when
    /// the write failed (the export fails too).
    pub async fn write(
        &self,
        export_id: ExportId,
        stream: ExportStream,
        position: u64,
        bytes: Vec<u8>,
    ) -> Result<(), AppError> {
        let (reply, written) = oneshot::channel();
        self.send(
            export_id,
            ExportMessage::Write {
                stream,
                position,
                bytes,
                reply,
            },
        )
        .await?;
        wait_written(export_id, written).await
    }

    /// Gives ffmpeg an uploaded RGBA frame, once it took it.
    ///
    /// # Errors
    /// As [`Self::write`].
    pub async fn write_frame(&self, export_id: ExportId, rgba: Vec<u8>) -> Result<(), AppError> {
        let (reply, written) = oneshot::channel();
        self.send(export_id, ExportMessage::Frame { rgba, reply })
            .await?;
        wait_written(export_id, written).await
    }

    async fn send(&self, export_id: ExportId, message: ExportMessage) -> Result<(), AppError> {
        let inbox = self.deps.registry.inbox(export_id)?;
        inbox
            .send(message)
            .await
            .map_err(|_| AppError::ExportNotRendering { export_id })
    }
}

async fn wait_written(
    export_id: ExportId,
    written: oneshot::Receiver<Result<(), MediaError>>,
) -> Result<(), AppError> {
    match written.await {
        Ok(outcome) => outcome.map_err(AppError::from),
        Err(_) => Err(AppError::ExportNotRendering { export_id }),
    }
}
