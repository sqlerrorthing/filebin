use std::error::Error;
use crate::error::sealed::Sealed;
use thiserror::Error;

/// The error can be either business or internal
#[derive(Debug, Error)]
pub enum ServiceError<B, I> {
    #[error("{0}")]
    Business(#[source] B),

    #[error("internal service error: {0}")]
    Internal(#[from] I),
}

#[macro_export]
macro_rules! business {
    ($($tt:tt)*) => {
        $crate::error::ServiceError::Business($($tt)*)
    };
}

#[macro_export]
macro_rules! internal {
    ($($tt:tt)*) => {
        $crate::error::ServiceError::Internal($($tt)*)
    };
}

mod sealed {
    pub trait Sealed {}
}

pub trait ResultExt<T, B, I>: Sealed {
    fn map_errs<BN, IN>(
        self,
        business: impl FnOnce(B) -> BN,
        internal: impl FnOnce(I) -> IN,
    ) -> Result<T, ServiceError<BN, IN>>;

    fn map_business<BN>(self, business: impl FnOnce(B) -> BN) -> Result<T, ServiceError<BN, I>>;

    fn map_internal<IN>(self, internal: impl FnOnce(I) -> IN) -> Result<T, ServiceError<B, IN>>;
}

impl<T, B, I> Sealed for Result<T, ServiceError<B, I>> {}

impl<T, B, I> ResultExt<T, B, I> for Result<T, ServiceError<B, I>> {
    fn map_errs<BN, IN>(
        self,
        business: impl FnOnce(B) -> BN,
        internal: impl FnOnce(I) -> IN,
    ) -> Result<T, ServiceError<BN, IN>> {
        self.map_err(|err| match err {
            ServiceError::Business(b) => business!(business(b)),
            ServiceError::Internal(i) => internal!(internal(i)),
        })
    }

    fn map_business<BN>(self, business: impl FnOnce(B) -> BN) -> Result<T, ServiceError<BN, I>> {
        self.map_err(|err| match err {
            ServiceError::Business(b) => business!(business(b)),
            ServiceError::Internal(i) => internal!(i),
        })
    }

    fn map_internal<IN>(self, internal: impl FnOnce(I) -> IN) -> Result<T, ServiceError<B, IN>> {
        self.map_err(|err| match err {
            ServiceError::Business(b) => business!(b),
            ServiceError::Internal(i) => internal!(internal(i)),
        })
    }
}

pub trait OptionExt<T> {
    fn ok_or_business<B: Error, I: Error>(self, err: B) -> Result<T, ServiceError<B, I>>;
    fn ok_or_else_business<B: Error, I: Error>(self, err: impl FnOnce() -> B) -> Result<T, ServiceError<B, I>>;
}

impl OptionExt<()> for bool {
    fn ok_or_business<B: Error, I: Error>(self, err: B) -> Result<(), ServiceError<B, I>> {
        self.ok_or_else_business(|| err)
    }

    fn ok_or_else_business<B: Error, I: Error>(self, err: impl FnOnce() -> B) -> Result<(), ServiceError<B, I>> {
        self.ok_or_else(|| business!(err()))
    }
}

impl<T> OptionExt<T> for Option<T> {
    #[inline(always)]
    fn ok_or_business<B: Error, I: Error>(self, err: B) -> Result<T, ServiceError<B, I>> {
        self.ok_or_else_business(|| err)
    }

    #[inline(always)]
    fn ok_or_else_business<B: Error, I: Error>(self, err: impl FnOnce() -> B) -> Result<T, ServiceError<B, I>> {
        self.ok_or_else(|| business!(err()))
    }
}