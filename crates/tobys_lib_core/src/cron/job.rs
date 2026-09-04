use ::core::{fmt, pin::Pin};
use jiff::civil::DateTime;

use crate::{
    alias::Box,
    cron::{CronParsingError, CronTime},
};

/// The future returned from a cron function.
pub type CronFuncFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(super) type AsyncFunction =
    dyn Fn() -> CronFuncFuture + Send + Sync + 'static;

/// A Cron Job
///
/// A cron job is the combination of a [`CronTime`] and the actual job (function) that needs to be executed
/// at the interval specified by the [`CronTime`]. It can be used with the [`Scheduler`] in [`Scheduler.add_job`],
/// to add it as a job the scheduler should schedule for evaluation.
///
/// [`Scheduler`]: crate::cron::Scheduler
/// [`Scheduler.add_job`]: crate::cron::Scheduler::add_job
///
/// # Example
/// Create a simple cron job that run all the time
/// ```rust
/// # use tobys_lib_core::cron::{CronTime, Job};
/// let v = Job::new(CronTime::new("* * * * *").expect("cron string is valid!"), || {
///     // prepare the async work..
///     Box::pin(async {
///         // do some async work ...
///     })
/// });
/// ```
pub struct Job {
    time: CronTime,
    pub(super) func: Box<AsyncFunction>,
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Job")
            .field("time", &self.time)
            .field("func", &"UNABLE_TO_DEBUG")
            .finish()
    }
}

impl Job {
    /// Creates a new asynchronous cron [`Job`] to be used in the scheduler.
    ///
    /// To allow async functions to work, you must return a pinned boxed future.
    /// See the example for how to.
    ///
    /// # Example
    /// Create a simple cron job that run all the time
    /// ```rust
    /// # use tobys_lib_core::cron::{CronTime, Job};
    /// let v = Job::new(CronTime::new("* * * * *").expect("cron string is valid!"), || {
    ///     // prepare the async work..
    ///     Box::pin(async {
    ///         // do some async work ...
    ///     })
    /// });
    /// ```
    pub fn new<F>(time: CronTime, func: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
            + Send
            + Sync
            + Copy
            + 'static,
    {
        Self {
            time,
            func: Box::new(func),
        }
    }

    /// Creates a new asynchronous cron [`Job`] to be used in the scheduler.
    ///
    /// This function can be used instead of [`new`]
    /// if you don't want to parse the [`CronTime`] yourself.
    ///
    /// [`new`]: Self::new
    ///
    /// # Errors
    ///
    /// Errors if the supplied cron string is invalid, see [`CronTime`] for a bigger explanation.
    ///
    /// # Example
    /// Create a simple cron job that run all the time
    /// ```rust
    /// # use tobys_lib_core::cron::{CronTime, Job};
    /// let v = Job::new_with_string("* * * * *", || {
    ///     // prepare the async work..
    ///     Box::pin(async {
    ///         // do some async work ...
    ///     })
    /// }).expect("cron string is valid!");
    /// ```
    pub fn new_with_string<F>(
        time: &str,
        func: F,
    ) -> Result<Self, CronParsingError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
            + Send
            + Sync
            + Copy
            + 'static,
    {
        CronTime::new(time).map(|time| Self {
            time,
            func: Box::new(func),
        })
    }

    #[must_use]
    pub(super) fn get_next_time(&self, curr: impl Into<DateTime>) -> DateTime {
        self.time.get_next_time(curr)
    }
}
