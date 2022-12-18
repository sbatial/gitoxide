use std::io;

use git_object::WriteTo;

/// Describe the capability to write git objects into an object store.
pub trait Write {
    /// The error type used for all trait methods.
    ///
    /// _Note_ the default implementations require the `From<io::Error>` bound.
    type Error: std::error::Error + From<io::Error>;

    /// Write objects using the intrinsic kind of [`hash`][git_hash::Kind] into the database,
    /// returning id to reference it in subsequent reads.
    fn write(&self, object: impl WriteTo) -> Result<git_hash::ObjectId, Self::Error> {
        let mut buf = Vec::with_capacity(2048);
        object.write_to(&mut buf)?;
        self.write_stream(object.kind(), buf.len() as u64, buf.as_slice())
    }
    /// As [`write`][Write::write], but takes an [`object` kind][git_object::Kind] along with its encoded bytes.
    fn write_buf(&self, object: git_object::Kind, from: &[u8]) -> Result<git_hash::ObjectId, Self::Error> {
        self.write_stream(object, from.len() as u64, from)
    }
    /// As [`write`][Write::write], but takes an input stream.
    /// This is commonly used for writing blobs directly without reading them to memory first.
    fn write_stream(
        &self,
        kind: git_object::Kind,
        size: u64,
        from: impl io::Read,
    ) -> Result<git_hash::ObjectId, Self::Error>;
}

/// Describe how object can be located in an object store.
///
/// ## Notes
///
/// Find effectively needs [generic associated types][issue] to allow a trait for the returned object type.
/// Until then, we will have to make due with explicit types and give them the potentially added features we want.
///
/// [issue]: https://github.com/rust-lang/rust/issues/44265
pub trait Find {
    /// The error returned by [`try_find()`][Find::try_find()]
    type Error: std::error::Error + 'static;

    /// Returns true if the object exists in the database.
    fn contains(&self, id: impl AsRef<git_hash::oid>) -> bool;

    /// Find an object matching `id` in the database while placing its raw, undecoded data into `buffer`.
    ///
    /// Returns `Some` object if it was present in the database, or the error that occurred during lookup or object
    /// retrieval.
    fn try_find<'a>(
        &self,
        id: impl AsRef<git_hash::oid>,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Option<git_object::Data<'a>>, Self::Error>;
}

/// A way to obtain object properties without fully decoding it.
pub trait Header {
    /// The error returned by [`try_header()`][Header::try_header()].
    type Error: std::error::Error + 'static;
    /// Try to read the header of the object associated with `id` or return `None` if it could not be found.
    fn try_header(&self, id: impl AsRef<git_hash::oid>) -> Result<Option<find::Header>, Self::Error>;
}

mod _impls {
    use std::{io::Read, ops::Deref, rc::Rc, sync::Arc};

    use crate::find::Header;
    use git_hash::{oid, ObjectId};
    use git_object::{Data, Kind, WriteTo};

    impl<T> crate::Write for &T
    where
        T: crate::Write,
    {
        type Error = T::Error;

        fn write(&self, object: impl WriteTo) -> Result<ObjectId, Self::Error> {
            (*self).write(object)
        }

        fn write_buf(&self, object: Kind, from: &[u8]) -> Result<ObjectId, Self::Error> {
            (*self).write_buf(object, from)
        }

        fn write_stream(&self, kind: Kind, size: u64, from: impl Read) -> Result<ObjectId, Self::Error> {
            (*self).write_stream(kind, size, from)
        }
    }

    impl<T> crate::Write for Arc<T>
    where
        T: crate::Write,
    {
        type Error = T::Error;

        fn write(&self, object: impl WriteTo) -> Result<ObjectId, Self::Error> {
            self.deref().write(object)
        }

        fn write_buf(&self, object: Kind, from: &[u8]) -> Result<ObjectId, Self::Error> {
            self.deref().write_buf(object, from)
        }

        fn write_stream(&self, kind: Kind, size: u64, from: impl Read) -> Result<ObjectId, Self::Error> {
            self.deref().write_stream(kind, size, from)
        }
    }

    impl<T> crate::Write for Rc<T>
    where
        T: crate::Write,
    {
        type Error = T::Error;

        fn write(&self, object: impl WriteTo) -> Result<ObjectId, Self::Error> {
            self.deref().write(object)
        }

        fn write_buf(&self, object: Kind, from: &[u8]) -> Result<ObjectId, Self::Error> {
            self.deref().write_buf(object, from)
        }

        fn write_stream(&self, kind: Kind, size: u64, from: impl Read) -> Result<ObjectId, Self::Error> {
            self.deref().write_stream(kind, size, from)
        }
    }

    impl<T> crate::Find for &T
    where
        T: crate::Find,
    {
        type Error = T::Error;

        fn contains(&self, id: impl AsRef<oid>) -> bool {
            (*self).contains(id)
        }

        fn try_find<'a>(&self, id: impl AsRef<oid>, buffer: &'a mut Vec<u8>) -> Result<Option<Data<'a>>, Self::Error> {
            (*self).try_find(id, buffer)
        }
    }

    impl<T> crate::Header for &T
    where
        T: crate::Header,
    {
        type Error = T::Error;

        fn try_header(&self, id: impl AsRef<oid>) -> Result<Option<Header>, Self::Error> {
            (*self).try_header(id)
        }
    }

    impl<T> crate::Find for Rc<T>
    where
        T: crate::Find,
    {
        type Error = T::Error;

        fn contains(&self, id: impl AsRef<oid>) -> bool {
            self.deref().contains(id)
        }

        fn try_find<'a>(&self, id: impl AsRef<oid>, buffer: &'a mut Vec<u8>) -> Result<Option<Data<'a>>, Self::Error> {
            self.deref().try_find(id, buffer)
        }
    }

    impl<T> crate::Header for Rc<T>
    where
        T: crate::Header,
    {
        type Error = T::Error;

        fn try_header(&self, id: impl AsRef<oid>) -> Result<Option<Header>, Self::Error> {
            self.deref().try_header(id)
        }
    }

    impl<T> crate::Find for Arc<T>
    where
        T: crate::Find,
    {
        type Error = T::Error;

        fn contains(&self, id: impl AsRef<oid>) -> bool {
            self.deref().contains(id)
        }

        fn try_find<'a>(&self, id: impl AsRef<oid>, buffer: &'a mut Vec<u8>) -> Result<Option<Data<'a>>, Self::Error> {
            self.deref().try_find(id, buffer)
        }
    }

    impl<T> crate::Header for Arc<T>
    where
        T: crate::Header,
    {
        type Error = T::Error;

        fn try_header(&self, id: impl AsRef<oid>) -> Result<Option<Header>, Self::Error> {
            self.deref().try_header(id)
        }
    }
}

mod ext {
    use git_object::{BlobRef, CommitRef, CommitRefIter, Kind, ObjectRef, TagRef, TagRefIter, TreeRef, TreeRefIter};

    use crate::find;

    macro_rules! make_obj_lookup {
        ($method:ident, $object_variant:path, $object_kind:path, $object_type:ty) => {
            /// Like [`find(…)`][Self::find()], but flattens the `Result<Option<_>>` into a single `Result` making a non-existing object an error
            /// while returning the desired object type.
            fn $method<'a>(
                &self,
                id: impl AsRef<git_hash::oid>,
                buffer: &'a mut Vec<u8>,
            ) -> Result<$object_type, find::existing_object::Error<Self::Error>> {
                let id = id.as_ref();
                self.try_find(id, buffer)
                    .map_err(find::existing_object::Error::Find)?
                    .ok_or_else(|| find::existing_object::Error::NotFound {
                        oid: id.as_ref().to_owned(),
                    })
                    .and_then(|o| o.decode().map_err(find::existing_object::Error::Decode))
                    .and_then(|o| match o {
                        $object_variant(o) => return Ok(o),
                        _other => Err(find::existing_object::Error::ObjectKind {
                            expected: $object_kind,
                        }),
                    })
            }
        };
    }

    macro_rules! make_iter_lookup {
        ($method:ident, $object_kind:path, $object_type:ty, $into_iter:tt) => {
            /// Like [`find(…)`][Self::find()], but flattens the `Result<Option<_>>` into a single `Result` making a non-existing object an error
            /// while returning the desired iterator type.
            fn $method<'a>(
                &self,
                id: impl AsRef<git_hash::oid>,
                buffer: &'a mut Vec<u8>,
            ) -> Result<$object_type, find::existing_iter::Error<Self::Error>> {
                let id = id.as_ref();
                self.try_find(id, buffer)
                    .map_err(find::existing_iter::Error::Find)?
                    .ok_or_else(|| find::existing_iter::Error::NotFound {
                        oid: id.as_ref().to_owned(),
                    })
                    .and_then(|o| {
                        o.$into_iter()
                            .ok_or_else(|| find::existing_iter::Error::ObjectKind {
                                expected: $object_kind,
                            })
                    })
            }
        };
    }

    /// An extension trait with convenience functions.
    pub trait HeaderExt: super::Header {
        /// Like [`try_header(…)`][super::Header::try_header()], but flattens the `Result<Option<_>>` into a single `Result` making a non-existing object an error.
        fn header(
            &self,
            id: impl AsRef<git_hash::oid>,
        ) -> Result<crate::find::Header, find::existing::Error<Self::Error>> {
            let id = id.as_ref();
            self.try_header(id)
                .map_err(find::existing::Error::Find)?
                .ok_or_else(|| find::existing::Error::NotFound { oid: id.to_owned() })
        }
    }

    impl<T: super::Header> HeaderExt for T {}

    /// An extension trait with convenience functions.
    pub trait FindExt: super::Find {
        /// Like [`try_find(…)`][super::Find::try_find()], but flattens the `Result<Option<_>>` into a single `Result` making a non-existing object an error.
        fn find<'a>(
            &self,
            id: impl AsRef<git_hash::oid>,
            buffer: &'a mut Vec<u8>,
        ) -> Result<git_object::Data<'a>, find::existing::Error<Self::Error>> {
            let id = id.as_ref();
            self.try_find(id, buffer)
                .map_err(find::existing::Error::Find)?
                .ok_or_else(|| find::existing::Error::NotFound { oid: id.to_owned() })
        }

        make_obj_lookup!(find_commit, ObjectRef::Commit, Kind::Commit, CommitRef<'a>);
        make_obj_lookup!(find_tree, ObjectRef::Tree, Kind::Tree, TreeRef<'a>);
        make_obj_lookup!(find_tag, ObjectRef::Tag, Kind::Tag, TagRef<'a>);
        make_obj_lookup!(find_blob, ObjectRef::Blob, Kind::Blob, BlobRef<'a>);
        make_iter_lookup!(find_commit_iter, Kind::Commit, CommitRefIter<'a>, try_into_commit_iter);
        make_iter_lookup!(find_tree_iter, Kind::Tree, TreeRefIter<'a>, try_into_tree_iter);
        make_iter_lookup!(find_tag_iter, Kind::Tag, TagRefIter<'a>, try_into_tag_iter);
    }

    impl<T: super::Find> FindExt for T {}
}
use crate::find;
pub use ext::{FindExt, HeaderExt};
