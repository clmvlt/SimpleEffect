//! [`NewIds`]: typed ids from the [`IdGenerator`] port.

use se_domain::{
    AssetId, ChangesetId, ItemId, JobId, LinkGroupId, MarkerId, ProjectId, SequenceId, TaskId,
    TrackId, TransactionId, TransitionId,
};

use crate::{AppError, ports::IdGenerator};

/// Creates typed ids from an [`IdGenerator`].
#[derive(Clone, Copy)]
pub struct NewIds<'a>(pub &'a dyn IdGenerator);

/// Declares one method per id type: the generator's UUID wrapped in that type.
macro_rules! id_methods {
    ($($(#[$doc:meta])* $method:ident -> $id:ty;)*) => {
        impl NewIds<'_> {
            $(
                $(#[$doc])*
                ///
                /// # Errors
                /// [`AppError::Internal`] if the generator broke its contract (not a UUIDv7).
                pub fn $method(self) -> Result<$id, AppError> {
                    <$id>::from_uuid(self.0.new_uuid_v7())
                        .map_err(|error| AppError::Internal(error.to_string()))
                }
            )*
        }
    };
}

id_methods! {
    /// A new project id.
    project -> ProjectId;
    /// A new sequence id.
    sequence -> SequenceId;
    /// A new track id.
    track -> TrackId;
    /// A new marker id.
    marker -> MarkerId;
    /// A new transaction id.
    transaction -> TransactionId;
    /// A new asset id.
    asset -> AssetId;
    /// A new background job id.
    job -> JobId;
    /// A new item id.
    item -> ItemId;
    /// A new link group id.
    link_group -> LinkGroupId;
    /// A new transition id.
    transition -> TransitionId;
    /// A new AI task id.
    task -> TaskId;
    /// A new changeset id (an AI task's first edit).
    changeset -> ChangesetId;
}
