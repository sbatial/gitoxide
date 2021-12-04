//!
mod error {
    use crate::easy;

    /// The error returned by [`commit(…)`][easy::Handle::commit()].
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        ReferenceNameValidation(#[from] git_ref::name::Error),
        #[error(transparent)]
        WriteObject(#[from] easy::object::write::Error),
        #[error(transparent)]
        ReferenceEdit(#[from] easy::reference::edit::Error),
    }
}
pub use error::Error;
