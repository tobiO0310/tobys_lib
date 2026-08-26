use ::core::pin::Pin;
use jiff::civil::DateTime;

use crate::cron::CronTime;

type AsyncFunction<'a> = dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    + Send
    + Sync
    + 'a;

/// A Cron Job
pub struct Job<'a> {
    time: CronTime,
    func: Box<AsyncFunction<'a>>,
}

impl<'a> Job<'a> {
    /// Creates a new asynchronous cron [`Job`] to be used in the scheduler.
    ///
    /// To allow async functions to work, you must return a pinned boxed future.
    /// See the example for how to.
    ///
    /// # Example
    /// Create a simple cron job that run all the time
    /// ```rust
    /// # use tobys_lib_core::cron::{CronTime, Job};
    /// let v = Job::new_async(CronTime::new("* * * * *").expect("yes!"), || {
    ///     // prepare the async work..
    ///     Box::pin(async {
    ///         // do some async work ...
    ///     })
    /// });
    /// ```
    pub fn new<F>(time: CronTime, func: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
            + Send
            + Sync
            + Copy
            + 'a,
    {
        Self {
            time,
            func: Box::new(func),
        }
    }

    #[must_use]
    pub(super) fn get_next_time(&self) -> DateTime {
        self.time.get_next_time()
    }

    pub(super) fn run_func(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            (self.func)().await;
        })
    }
}
