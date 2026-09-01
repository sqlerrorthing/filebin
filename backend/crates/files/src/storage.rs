use std::cell::UnsafeCell;
use bytes::Bytes;
use domain::models::files;
use futures_core::Stream;
use serde::Serialize;
use serde::de::DeserializeOwned;
use service::service;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use parking_lot::{Mutex, MutexGuard, RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod s3;

/// Contains files prefix.
/// Should be used with `{FILES_PREFIX}/{key}`
pub const FILES_PREFIX: &str = "files";

#[service]
pub trait FilesStorage {
    type Error;

    type MultipartUploadHandle: MultipartUploadHandle;

    type GetFileStream: Stream<Item = Result<Bytes, Self::Error>> + Debug;

    /// Creates new mulipart upload stream
    ///
    /// Returns the upload handler, when it drops its automatically calls [`FilesStorage::abort_multipart_upload`]
    #[result]
    async fn create_multipart_upload(&self, key: files::StoragePath)
    -> Self::MultipartUploadHandle;

    #[result]
    async fn upload_part(
        &self,
        handle: &impl LockRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >,
        part: Bytes,
    );

    /// Completes the multipart upload
    ///
    /// Returns the key
    #[result]
    async fn complete_multipart_upload<H>(
        &self,
        handle: H,
    ) -> files::StoragePath
    where
        H: IntoRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >;

    /// Bulk deletes the provided ids
    #[result]
    async fn bulk_delete(&self, ids: Vec<files::StoragePath>);

    /// Deletes only one provided file
    #[result]
    async fn delete(&self, id: files::StoragePath);

    #[result]
    async fn get_file(&self, path: files::StoragePath) -> Option<Self::GetFileStream>;
}

pub trait RawMultipartUploadHandle:
    Send + Sync + Serialize + DeserializeOwned + Debug + Clone + 'static
{
}

impl<T: Send + Sync + Serialize + DeserializeOwned + Debug + Clone + 'static> RawMultipartUploadHandle
    for T
{
}

macro_rules! lock_impls {
    ($(
        $ty:ty: (|$read_ty:ty as $read_self:ident| $read:expr, |$write_ty:ty as $write_self:ident| $write:expr), (|$into_self:ident| $into:expr, |$from_raw:ident| $from:expr);
    )*) => {
        $(
            impl<T: RawMultipartUploadHandle> HasRawMultipartUploadHandle for $ty {
                type Raw = T;
            }

            impl<T: RawMultipartUploadHandle> IntoRawMultipartUploadHandle for $ty {
                type Rest = ();

                async fn into_raw(self) -> Option<(Self::Raw, Self::Rest)> {
                    let $into_self = self;
                    Some(($into, ()))
                }

                fn from_raw($from_raw: Self::Raw, _rest: Self::Rest) -> Self {
                    $from
                }
            }

            impl<T: RawMultipartUploadHandle> LockRawMultipartUploadHandle for $ty {
                type ReadRaw<'a> = $read_ty;
                type WriteRaw<'a> = $write_ty;

                async fn read_raw(&self) -> Option<Self::ReadRaw<'_>> {
                    let $read_self = self;
                    Some($read)
                }

                async fn write_raw(&self) -> Option<Self::WriteRaw<'_>> {
                    let $write_self = self;
                    Some($write)
                }
            }
        )*
    };
}


lock_impls! {
    RwLock<T>: (
        |RwLockReadGuard<'a, T> as this| this.read(),
        |RwLockWriteGuard<'a, T> as this| this.write()
    ), (
        |this| this.into_inner(),
        |raw| Self::new(raw)
    );

    Mutex<T>: (
        |MutexGuard<'a, T> as this| this.lock(),
        |MutexGuard<'a, T> as this| this.lock()
    ), (
        |this| this.into_inner(),
        |raw| Self::new(raw)
    );
}

pub trait HasRawMultipartUploadHandle {
    type Raw: RawMultipartUploadHandle;
}

pub trait IntoRawMultipartUploadHandle: LockRawMultipartUploadHandle + Send + Sync + 'static {
    type Rest: Send;

    /// Consumes the handle, returning both the serializable raw data
    /// and its internal resources (such as the S3 client) needed for [`from_raw`](Self::from_raw)
    ///
    /// Returns None if handle is dropped
    fn into_raw(self) -> impl Future<Output = Option<(Self::Raw, Self::Rest)>> + Send;

    // Re-creates the RAII handle from its raw data and internal
    fn from_raw(raw: Self::Raw, rest: Self::Rest) -> Self;
}

pub trait LockRawMultipartUploadHandle: HasRawMultipartUploadHandle + Send + Sync + 'static {
    type ReadRaw<'a>: 'a + Deref<Target = Self::Raw>;
    type WriteRaw<'a>: 'a + DerefMut<Target = Self::Raw>;

    /// Returns a reference to the raw handle (which might be protected by a mutex guard)
    ///
    /// This can lock the inner synchronization primitives.
    /// Prefer using [`with_read`](Self::with_read) for short-lived scopes to avoid deadlocks or extended lock contention
    ///
    /// Returns None if handle is dropped
    fn read_raw<'a>(&'a self) -> impl Future<Output = Option<Self::ReadRaw<'a>>> + Send + 'a;

    /// Returns a mutable reference to the raw handle (which might be protected by a mutex guard)
    ///
    /// This can lock the inner synchronization primitives.
    /// Prefer using [`with_write`](Self::with_write) for short-lived scopes.
    ///
    /// Returns None if handle is dropped
    fn write_raw<'a>(&'a self) -> impl Future<Output = Option<Self::WriteRaw<'a>>> + Send + 'a;

    /// Executes a closure with a borrowed reference to the raw handle,
    /// automatically dropping any internal locks (like mutex guards) immediately afterward
    ///
    /// Returns `None` if the underlying handle has already been consumed
    fn with_read<'a, R>(&'a self, f: impl FnOnce(&Self::Raw) -> R + Send + 'a) -> impl Future<Output = Option<R>> + Send + 'a {
        async move {
            self.read_raw().await.map(|r| f(r.deref()))
        }
    }

    /// Executes a closure with a mutable reference to the raw handle,
    /// automatically dropping any internal locks immediately afterward
    ///
    /// Returns `None` if the underlying handle has already been consumed
    fn with_write<'a, R>(&'a self, f: impl FnOnce(&mut Self::Raw) -> R + Send + 'a) -> impl Future<Output = Option<R>> + Send + 'a {
        async move {
            self.write_raw().await.map(|mut r| f(r.deref_mut()))
        }
    }
}

pub trait MultipartUploadHandle: LockRawMultipartUploadHandle + IntoRawMultipartUploadHandle {}

impl<T> MultipartUploadHandle for T where
    T: LockRawMultipartUploadHandle + IntoRawMultipartUploadHandle
{
}
