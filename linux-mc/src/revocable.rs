use core::{
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// Representation of an Object that might get revoked even if still allocated
///
/// # Examples
///
/// ```
/// use std::ops::Deref;
/// use linux_mc::Revocable;
///
/// let val = Revocable::new(2);
/// let guard = val.try_access().unwrap();
/// assert_eq!(*guard, 2);
/// ```
///
/// ```
/// use linux_mc::Revocable;
///
/// let val = Revocable::new(2);
/// assert!(val.try_access().is_some());
///
/// val.revoke();
/// assert!(val.try_access().is_none());
/// ```
#[derive(Debug)]
pub struct Revocable<T> {
    is_available: AtomicBool,
    content: T,
}

impl<T> Revocable<T> {
    /// Creates a new Revocable object
    pub fn new(obj: T) -> Self {
        Self {
            is_available: AtomicBool::new(true),
            content: obj,
        }
    }

    /// Tries to access the object.
    ///
    /// Returns None if the object is no longer accessible. Returns a guard that gives access to the
    /// object otherwise.
    pub fn try_access(&self) -> Option<RevocableGuard<'_, T>> {
        if self.is_available.load(Ordering::Relaxed) {
            Some(RevocableGuard {
                content_ref: &self.content,
            })
        } else {
            None
        }
    }

    /// Tries to access the object.
    ///
    /// Returns None if the object is no longer accessible. Returns a guard that gives access to the
    /// object otherwise.
    pub fn try_access_mut(&mut self) -> Option<RevocableGuardMut<'_, T>> {
        if self.is_available.load(Ordering::Relaxed) {
            Some(RevocableGuardMut {
                content_ref: &mut self.content,
            })
        } else {
            None
        }
    }

    /// Revokes access to the object
    pub fn revoke(&self) {
        self.is_available.store(false, Ordering::Relaxed);
    }
}

/// A guard that allows access to a revocable object.
#[derive(Debug)]
pub struct RevocableGuard<'a, T> {
    content_ref: &'a T,
}

impl<T> Deref for RevocableGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.content_ref
    }
}

/// A guard that allows mutable access to a revocable object.
#[derive(Debug)]
pub struct RevocableGuardMut<'a, T> {
    content_ref: &'a mut T,
}

impl<T> Deref for RevocableGuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.content_ref
    }
}
impl<T> DerefMut for RevocableGuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.content_ref
    }
}

/// Unwraps an `Option<RevocableGuard<T>>` or returns `RevocableValue::Revoked` if None.
///
/// # Examples
///
/// ```
/// use linux_mc::{Revocable, RevocableValue, try_option_to_value};
///
/// fn add_two(val: Revocable<u32>) -> RevocableValue<u32> {
///     let inner = try_option_to_value!(val.try_access());
///
///     RevocableValue::Value(*inner + 2)
/// }
///
/// assert_eq!(add_two(Revocable::new(2)), RevocableValue::Value(4));
///
/// let revoked = Revocable::new(2);
/// revoked.revoke();
/// assert_eq!(add_two(revoked), RevocableValue::Revoked);
/// ```
#[macro_export]
macro_rules! try_option_to_value {
    ($a:expr) => {
        match $a {
            Some(v) => v,
            None => return RevocableValue::Revoked.into(),
        }
    };
}

/// Unwraps an `Option<RevocableGuard<T>>` or returns `RevocableResult::Revoked` if None.
///
/// # Examples
///
/// ```
/// use linux_mc::{Revocable, RevocableResult, try_option_to_result};
///
/// fn add_two(val: Revocable<u32>) -> RevocableResult<u32, ()> {
///     let inner = try_option_to_result!(val.try_access());
///
///     RevocableResult::Ok(*inner + 2)
/// }
///
/// assert_eq!(add_two(Revocable::new(2)), RevocableResult::Ok(4));
///
/// let revoked = Revocable::new(2);
/// revoked.revoke();
/// assert_eq!(add_two(revoked), RevocableResult::Revoked);
/// ```
#[macro_export]
macro_rules! try_option_to_result {
    ($a:expr) => {
        match $a {
            Some(v) => v,
            None => return RevocableResult::Revoked.into(),
        }
    };
}

/// Represents a returned value from an object that is valid or has been revoked
#[must_use]
#[derive(Debug, PartialEq)]
pub enum RevocableValue<T> {
    /// The Object we were accessing was valid, and we contain the expected value.
    Value(T),

    /// The object we were accessing has been revoked.
    Revoked,
}

impl<T> RevocableValue<T> {
    /// Maps a `RevocableValue<T>` to `RevocableValue<U>` by applying a function to a contained
    /// Value value, ignoring the revoked case.
    ///
    /// # Examples
    ///
    /// ```
    /// use linux_mc::RevocableValue;
    ///
    /// let val: RevocableValue<u32> = RevocableValue::Value(2);
    /// assert_eq!(val.map(|v| v * 2), RevocableValue::Value(4));
    ///
    /// let val: RevocableValue<u32> = RevocableValue::Revoked;
    /// assert_eq!(val.map(|v| v * 2), RevocableValue::Revoked);
    /// ```
    pub fn map<U, F>(self, f: F) -> RevocableValue<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Value(v) => RevocableValue::Value(f(v)),
            Self::Revoked => RevocableValue::Revoked,
        }
    }

    /// Returns the contained `Value` value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Err` or `Revoked`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use linux_mc::RevocableValue;
    ///
    /// let v: RevocableValue<_> = RevocableValue::Value(2);
    /// assert_eq!(v.unwrap(), 2);
    /// ```
    ///
    /// ```should_panic
    /// use linux_mc::RevocableValue;
    ///
    /// let v: RevocableValue<()> = RevocableValue::Revoked;
    /// v.unwrap(); // panics
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            RevocableValue::Value(v) => v,
            RevocableValue::Revoked => {
                panic!("Called `RevocableValue::unwrap()` on a `Revoked` value")
            }
        }
    }

    /// Turns a `RevocableResult<T,E>` into a `Result<T,E>`, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Revoked`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use linux_mc::RevocableValue;
    ///
    /// let v = RevocableValue::Value(2);
    /// assert_eq!(v.valid(), 2);
    /// ```
    ///
    /// ```should_panic
    /// use linux_mc::RevocableValue;
    ///
    /// let v: RevocableValue<()> = RevocableValue::Revoked;
    /// v.valid(); // panics
    /// ```
    pub fn valid(self) -> T {
        match self {
            RevocableValue::Value(v) => v,
            RevocableValue::Revoked => {
                panic!("Called `RevocableValue::valid()` on a `Revoked` value")
            }
        }
    }
}

impl<T> fmt::Display for RevocableValue<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RevocableValue::Value(v) => v.fmt(f),
            RevocableValue::Revoked => f.write_str("(Revoked)"),
        }
    }
}

/// Unwraps a revocable value or returns if revoked.
///
/// # Examples
///
/// ```
/// use linux_mc::{RevocableValue, try_value};
///
/// fn add_two(res: RevocableValue<u32>) -> RevocableValue<u32> {
///     RevocableValue::Value(try_value!(res) + 2)
/// }
///
/// assert_eq!(add_two(RevocableValue::Value(2)), RevocableValue::Value(4));
/// assert_eq!(add_two(RevocableValue::Revoked), RevocableValue::Revoked);
/// ```
#[macro_export]
macro_rules! try_value {
    ($a:expr) => {
        match $a {
            RevocableValue::Value(v) => v,
            RevocableValue::Revoked => return RevocableValue::Revoked.into(),
        }
    };
}

/// Represents the returned value of a revocable object that can be either a success value or an
/// error value if the object was still valid, or Revoked if it wasn't.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub enum RevocableResult<T, E> {
    /// The Object we were accessing was valid, and the operation was successful.
    Ok(T),

    /// The object we were accessing has been revoked.
    Revoked,

    /// The Object we were accessing was valid, but the operation failed.
    Err(E),
}

impl<T, E> RevocableResult<T, E> {
    /// Maps a `RevocableResult<T, E>` to `RevocableResult<U>` by applying a function to a contained
    /// Value value, leaving the Error value untouched, and ignoring the revoked case.
    ///
    /// # Examples
    ///
    /// ```
    /// use linux_mc::RevocableResult;
    ///
    /// let val: RevocableResult<u32, u32> = RevocableResult::Ok(2);
    /// assert_eq!(val.map(|v| v * 2), RevocableResult::Ok(4));
    ///
    /// let val: RevocableResult<u32, u32> = RevocableResult::Revoked;
    /// assert_eq!(val.map(|v| v * 2), RevocableResult::Revoked);
    ///
    /// let val: RevocableResult<u32, u32> = RevocableResult::Err(2);
    /// assert_eq!(val.map(|v| v * 2), RevocableResult::Err(2));
    /// ```
    pub fn map<U, F>(self, f: F) -> RevocableResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Ok(v) => RevocableResult::Ok(f(v)),
            Self::Revoked => RevocableResult::Revoked,
            Self::Err(e) => RevocableResult::Err(e),
        }
    }

    /// Returns the contained `Ok` value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Err` or `Revoked`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use linux_mc::RevocableResult;
    ///
    /// let v: RevocableResult<_, ()> = RevocableResult::Ok(2);
    /// assert_eq!(v.unwrap(), 2);
    /// ```
    ///
    /// ```should_panic
    /// use linux_mc::RevocableResult;
    ///
    /// let v: RevocableResult<(), _> = RevocableResult::Err(42);
    /// v.unwrap(); // panics
    /// ```
    ///
    /// ```should_panic
    /// use linux_mc::RevocableResult;
    ///
    /// let v: RevocableResult<(), ()> = RevocableResult::Revoked;
    /// v.unwrap(); // panics
    /// ```
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            RevocableResult::Ok(v) => v,
            RevocableResult::Revoked => {
                panic!("Called `RevocableResult::unwrap()` on a `Revoked` value")
            }
            RevocableResult::Err(e) => {
                panic!("Called `RevocableResult::unwrap()` on a `Err` value: {e:?}")
            }
        }
    }

    /// Turns a `RevocableResult<T,E>` into a `Result<T,E>`, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Revoked`
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use linux_mc::RevocableResult;
    ///
    /// let v: RevocableResult<_, ()> = RevocableResult::Ok(2);
    /// assert_eq!(v.valid(), Ok(2));
    ///
    /// let v: RevocableResult<(), _> = RevocableResult::Err(42);
    /// assert_eq!(v.valid(), Err(42));
    /// ```
    ///
    /// ```should_panic
    /// use linux_mc::RevocableResult;
    ///
    /// let v: RevocableResult<(), ()> = RevocableResult::Revoked;
    /// v.valid(); // panics
    /// ```
    #[expect(
        clippy::missing_errors_doc,
        clippy::panic_in_result_fn,
        reason = "It's kind of the whole point."
    )]
    pub fn valid(self) -> Result<T, E> {
        match self {
            RevocableResult::Ok(v) => Ok(v),
            RevocableResult::Revoked => {
                panic!("Called `RevocableResult::valid()` on a `Revoked` value")
            }
            RevocableResult::Err(e) => Err(e),
        }
    }
}

impl<T, E> From<RevocableValue<T>> for RevocableResult<T, E> {
    fn from(value: RevocableValue<T>) -> Self {
        match value {
            RevocableValue::Value(v) => RevocableResult::Ok(v),
            RevocableValue::Revoked => RevocableResult::Revoked,
        }
    }
}

impl<T, E> From<Result<T, E>> for RevocableResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => RevocableResult::Ok(v),
            Err(e) => RevocableResult::Err(e),
        }
    }
}

/// Unwraps a revocable value or returns if revoked or failure.
///
/// # Examples
///
/// ```
/// use linux_mc::{RevocableResult, try_result};
///
/// fn add_two(res: RevocableResult<u32, ()>) -> RevocableResult<u32, ()> {
///     RevocableResult::Ok(try_result!(res) + 2)
/// }
///
/// assert_eq!(add_two(RevocableResult::Ok(2)), RevocableResult::Ok(4));
/// assert_eq!(add_two(RevocableResult::Revoked), RevocableResult::Revoked);
/// assert_eq!(add_two(RevocableResult::Err(())), RevocableResult::Err(()));
/// ```
#[macro_export]
macro_rules! try_result {
    ($a:expr) => {
        match $a {
            RevocableResult::Ok(v) => v,
            RevocableResult::Revoked => return RevocableResult::Revoked.into(),
            RevocableResult::Err(e) => return RevocableResult::Err(e).into(),
        }
    };
}

/// Unwraps a `Result::Ok` value or propagates its error as a `RevocableResult`
///
/// # Examples
///
/// ```
/// use linux_mc::{RevocableResult, try_result_to_revocable};
///
/// fn add_two(res: Result<u32, ()>) -> RevocableResult<u32, ()> {
///     RevocableResult::Ok(try_result_to_revocable!(res) + 2)
/// }
///
/// assert_eq!(add_two(Ok(2)), RevocableResult::Ok(4));
/// assert_eq!(add_two(Err(())), RevocableResult::Err(()));
/// ```
#[macro_export]
macro_rules! try_result_to_revocable {
    ($a: expr) => {
        match $a {
            Ok(v) => v,
            Err(e) => return RevocableResult::Err(e.into()).into(),
        }
    };
}
