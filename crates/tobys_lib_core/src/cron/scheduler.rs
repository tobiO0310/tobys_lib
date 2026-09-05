use core::marker::PhantomData;

use ::core::cmp::Ordering;
#[cfg(feature = "std")]
use jiff::Zoned;
use jiff::civil::DateTime;

use crate::{
    alias::Vec,
    cron::{CronFuncFuture, Job},
};

#[derive(Debug)]
struct JobOrdering(Job, DateTime);
impl PartialEq for JobOrdering {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for JobOrdering {}
impl PartialOrd for JobOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JobOrdering {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

/// A [`Scheduler`] that has this type is not initialized.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uninitialized;
/// A [`Scheduler`] that has this type is initialized.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Initialized;

/// The scheduler for cron [`Job`]s.
///
/// The scheduler is responsible for the scheduling and---possible---execution of cron [`Job`]s.
/// The reason for it being possible is supporting parallelization in any async runtime,
/// by giving the user the futures that needs to be awaited.
/// This separation can be seen in [`Scheduler.tick`] and [`Scheduler.manual_tick`].
/// The prior doesn't return any futures (except its own) that can be separately put into an async runtime,
/// whereas the later does return a [`Vec`] of [`CronFuncFuture`]s.
///
///
///
/// [`Scheduler.tick`]: Self::tick
/// [`Scheduler.manual_tick`]: Self::manual_tick
///
/// # no-std support
///
/// Some functions are listed as "std feature only", this is simply because rustdoc uses all features.
/// The functions marked as "std feature required" have no-std variants.
///
/// The reason for this disparity, is that in a no-std environment, [`jiff::Zoned::now`] is unavailable.
/// Therefore it must be up to the user themselves to supply a [`DateTime`] equivalent.
///
/// # Examples
/// Create a scheduler and begin running it
/// ```rust
/// # use tobys_lib_core::cron::{Job, Scheduler};
/// # use futures::future::join_all;
/// # use std::{time::Duration, thread::sleep};
/// # #[tokio::main]
/// # async fn main() {
/// let mut scheduler = Scheduler::new();
/// let job = Job::new_with_string("* * * * *", || {
///     Box::pin(async {
///         println!("Work has been done!");
///     })
/// }).expect("is valid cron string");
/// scheduler.add_job(job);
/// let mut scheduler = scheduler.init();
/// tokio::task::spawn(async move {
///     let duration = Duration::from_secs(29);
///     loop {
///         join_all(scheduler.manual_tick()).await;
///         sleep(duration);
///         # break;
///     }
/// });
/// # }
/// ```
#[derive(Debug)]
#[must_use = "The scheduler is lazy and does nothing on its own without using it"]
pub struct Scheduler<S> {
    scheduled_jobs: Vec<Job>,
    upcoming_jobs: Vec<JobOrdering>,
    status: PhantomData<S>,
}
impl<S> Default for Scheduler<S> {
    fn default() -> Self {
        Self {
            scheduled_jobs: Vec::default(),
            upcoming_jobs: Vec::default(),
            status: PhantomData,
        }
    }
}

impl Scheduler<Uninitialized> {
    /// Creates a new [`Scheduler`] for cron [`Job`]s.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Scheduler`] and adds all [`Job`]s in the iterator.
    ///
    /// This will preallocate the underlying data structure,
    /// adding more jobs later will cause them to reallocate.
    ///
    /// # Example
    /// ```rust
    /// # use tobys_lib_core::cron::{Scheduler};
    /// # use tobys_lib::create_cron_jobs;
    /// let mut scheduler = Scheduler::new_with_jobs(create_cron_jobs!(
    ///     * * * * *, || {
    ///         Box::pin(async {
    ///             println!("async work...");
    ///         })
    ///     }
    /// ));
    pub fn new_with_jobs<I: IntoIterator<Item = Job>>(jobs: I) -> Self {
        let scheduled_jobs: Vec<_> = jobs.into_iter().collect();
        let len = scheduled_jobs.len();

        Self {
            scheduled_jobs,
            upcoming_jobs: Vec::with_capacity(len),
            status: PhantomData,
        }
    }

    /// Add a [`Job`] to the [`Scheduler`].
    pub fn add_job(&mut self, job: Job) {
        self.scheduled_jobs.push(job);
    }

    fn __init(self, curr: impl Into<DateTime>) -> Scheduler<Initialized> {
        let curr = curr.into();
        let mut upcoming_jobs: Vec<_> = self
            .scheduled_jobs
            .into_iter()
            .map(|v| {
                let next = v.get_next_time(curr);
                JobOrdering(v, next)
            })
            .collect();
        upcoming_jobs.sort();

        Scheduler {
            scheduled_jobs: Vec::default(),
            upcoming_jobs,
            status: PhantomData,
        }
    }

    /// Initializes the scheduler.
    ///
    /// Because this is a no-std environment, you must manually give the current time,
    /// using whatever hardware you have available.
    ///
    /// This function prepares the scheduler for running,
    /// and must be called before any ticking can be done.
    #[cfg(not(feature = "std"))]
    pub fn init(self, curr: impl Into<DateTime>) -> Scheduler<Initialized> {
        self.__init(curr)
    }

    /// Initializes the scheduler.
    ///
    /// This function prepares the scheduler for running,
    /// and must be called before any ticking can be done.
    #[cfg(feature = "std")]
    pub fn init(self) -> Scheduler<Initialized> {
        self.__init(Zoned::now())
    }
}

impl Scheduler<Initialized> {
    /// Add a [`Job`] to the [`Scheduler`].
    ///
    /// Because this is a no-std environment, you must manually give the current time,
    /// using whatever hardware you have available.
    #[cfg(not(feature = "std"))]
    pub fn add_job(&mut self, job: Job, curr: impl Into<DateTime>) {
        let next_time = job.get_next_time(curr);
        let job = JobOrdering(job, next_time);
        match self.upcoming_jobs.binary_search(&job) {
            Ok(index) | Err(index) => {
                self.upcoming_jobs.insert(index, job);
            }
        }
    }

    /// Add a [`Job`] to the [`Scheduler`].
    #[cfg(feature = "std")]
    pub fn add_job(&mut self, job: Job) {
        let curr = Zoned::now();
        let next_time = job.get_next_time(curr);
        let job = JobOrdering(job, next_time);
        match self.upcoming_jobs.binary_search(&job) {
            Ok(index) | Err(index) => {
                self.upcoming_jobs.insert(index, job);
            }
        }
    }

    /// Executes the cron jobs to be executed at this point
    ///
    /// Because this is a no-std environment, you must manually give the current time,
    /// using whatever hardware you have available.
    ///
    /// This will tick each job once every-time the async runtime
    /// allows this function's future to continue.
    /// If you wish to parallelize it, e.g. via [`futures::future::join_all`],
    /// you should use [`Self.manual_tick`].
    ///
    /// [`Self.manual_tick`]: Self::manual_tick
    /// # Errors
    ///
    /// This will error if the [`Scheduler`] is uninitialized.
    #[cfg(not(feature = "std"))]
    #[inline]
    pub async fn tick(&mut self, now: impl Into<DateTime>) {
        for job in self.manual_tick(now) {
            job.await;
        }
    }

    /// Executes the cron jobs to be executed at this point
    ///
    /// This will tick each job once every-time the async runtime
    /// allows this function's future to continue.
    /// If you wish to parallelize it, using your async runtime,
    /// you should use [`Self.manual_tick`].
    ///
    /// [`Self.manual_tick`]: Self::manual_tick
    /// # Errors
    ///
    /// This will error if the [`Scheduler`] is uninitialized.
    #[cfg(feature = "std")]
    #[inline]
    pub async fn tick(&mut self) {
        for job in self.manual_tick() {
            job.await;
        }
    }

    fn __manual_tick(
        &mut self,
        now: impl Into<DateTime>,
    ) -> Vec<CronFuncFuture> {
        let now = now.into();

        #[cfg(debug_assertions)]
        let total_jobs = self.upcoming_jobs.len();

        let jobs_to_run = self.upcoming_jobs.extract_if(.., |v| v.1 <= now);
        let mut futures = Vec::new();
        let mut finished_jobs = Vec::new();
        for job in jobs_to_run {
            futures.push((job.0.func)());
            finished_jobs.push(job);
        }
        // total jobs should be jobs left in queue, plus those added to "finished_jobs"
        debug_assert_eq!(
            total_jobs,
            self.upcoming_jobs.len().saturating_add(finished_jobs.len())
        );
        for v in &mut finished_jobs {
            let JobOrdering(job, _) = v;
            v.1 = job.get_next_time(now);
        }
        self.upcoming_jobs.append(&mut finished_jobs);
        self.upcoming_jobs.sort();
        // even though stability is not required,
        // the slice should be two blocks which are fully sorted: "upcoming_jobs" + "finished_jobs"

        // no job should've been deleted or scheduled twice
        debug_assert_eq!(total_jobs, self.upcoming_jobs.len());

        futures
    }

    /// Returns the cron jobs to be executed.
    ///
    /// Because this is a no-std environment, you must manually give the current time,
    /// using whatever hardware you have available.
    ///
    /// This is useful when trying to parallelize cron job execution.
    /// Should you not care about it, you could use [`Scheduler.tick`]
    ///
    /// You should strive to call this function and run this function at least every minute.
    ///
    /// ```rust
    /// # use tobys_lib_core::cron::Scheduler;
    /// # use futures::future::join_all;
    /// # use std::{thread::sleep, time::Duration};
    /// # let rt  = tokio::runtime::Runtime::new().unwrap();
    /// # fn time_now() -> jiff::civil::DateTime { jiff::civil::DateTime::default() }
    /// let mut scheduler = Scheduler::new();
    /// // add jobs... via .add_job
    /// scheduler.init();
    /// rt.block_on(async move {
    ///     let mut scheduler = scheduler;
    ///     loop {
    ///         // scheduler is initialized, error won't happen
    ///         let jobs = scheduler.manual_tick(time_now()).unwrap();
    ///         join_all(jobs).await;
    ///         // run every half minute (minus a sec for cpu variation)
    ///         sleep(Duration::from_secs(29));
    ///     }
    /// });
    /// ```
    #[cfg(not(feature = "std"))]
    pub fn manual_tick(
        &mut self,
        now: impl Into<DateTime>,
    ) -> Vec<CronFuncFuture> {
        self.__manual_tick(now)
    }

    /// Returns the cron jobs to be executed at this point.
    ///
    /// This is useful when trying to parallelize cron job execution.
    /// Should you not care about it, you could use [`Scheduler.tick`].
    ///
    /// You should strive to call this function and run this function at least every minute.
    ///
    /// ```rust
    /// # use tobys_lib_core::cron::Scheduler;
    /// # use futures::future::join_all;
    /// # use std::{thread::sleep, time::Duration};
    /// # let rt  = tokio::runtime::Runtime::new().unwrap();
    /// let mut scheduler = Scheduler::new();
    /// // add jobs... via .add_job
    /// let scheduler = scheduler.init();
    /// rt.block_on(async move {
    ///     let mut scheduler = scheduler;
    ///     loop {
    ///         // scheduler is initialized, error won't happen
    ///         let jobs = scheduler.manual_tick();
    ///         join_all(jobs).await;
    ///         // run every half minute (minus a sec for cpu variation)
    ///         sleep(Duration::from_secs(29));
    ///         # break;
    ///     }
    /// });
    /// ```
    ///
    /// [`Scheduler.tick`]: Self::tick
    #[cfg(feature = "std")]
    pub fn manual_tick(&mut self) -> Vec<CronFuncFuture> {
        self.__manual_tick(Zoned::now())
    }
}

#[cfg(feature = "std")]
impl Extend<Job> for Scheduler<Uninitialized> {
    fn extend<T: IntoIterator<Item = Job>>(&mut self, iter: T) {
        for job in iter {
            self.add_job(job);
        }
    }
}
#[cfg(feature = "std")]
impl Extend<Job> for Scheduler<Initialized> {
    fn extend<T: IntoIterator<Item = Job>>(&mut self, iter: T) {
        for job in iter {
            self.add_job(job);
        }
    }
}
#[cfg(not(feature = "std"))]
impl Extend<Job> for Scheduler<Uninitialized> {
    fn extend<I: IntoIterator<Item = Job>>(&mut self, iter: I) {
        for job in iter {
            self.add_job(job);
        }
    }
}
#[cfg(not(feature = "std"))]
impl<T: Into<DateTime>> Extend<(Job, T)> for Scheduler<Initialized> {
    fn extend<I: IntoIterator<Item = (Job, T)>>(&mut self, iter: I) {
        for (job, time) in iter {
            self.add_job(job, time);
        }
    }
}
impl FromIterator<Job> for Scheduler<Uninitialized> {
    fn from_iter<T: IntoIterator<Item = Job>>(iter: T) -> Self {
        Self::new_with_jobs(iter)
    }
}

#[cfg(feature = "std")] // will require std to actually do some async work here lol
#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use tokio::runtime::Runtime;

    use super::*;
    use crate::cron::CronTime;

    #[test]
    fn basic_test() {
        let rt = Runtime::new().unwrap();
        let mut scheduler = Scheduler::new();
        let cron_time = CronTime::new("* * * * *").unwrap();
        scheduler.add_job(Job::new(cron_time, || {
            Box::pin(async {
                println!("testing");
            })
        }));

        let scheduler = scheduler.init();
        let mut now = Zoned::now().datetime();
        let duration = cron_time
            .get_next_time(now)
            .duration_since(now)
            .unsigned_abs();
        now += duration; // move to next minute
        rt.block_on(async move {
            let mut scheduler = scheduler;

            // scheduler is initialized, error won't happen
            let jobs = scheduler.__manual_tick(now);
            assert_eq!(jobs.len(), 1); // the added println job must run now
            join_all(jobs).await;
        });
    }

    #[test]
    fn manual_tick_test() {
        let rt = Runtime::new().unwrap();
        let now = Zoned::now();
        let every = CronTime::new("* * * * *").unwrap();

        let scheduler = Scheduler::new_with_jobs([
            Job::new_with_string("* * * * *", || {
                Box::pin(async {
                    println!("every minute (*)");
                })
            })
            .unwrap(),
            Job::new_with_string(
                &format!("{} * * * *", (now.minute() + 1) % 60),
                || {
                    Box::pin(async {
                        println!("next minute (+1)");
                    })
                },
            )
            .unwrap(),
            Job::new_with_string(
                &format!("{} * * * *", (now.minute() + 3) % 60),
                || {
                    Box::pin(async {
                        println!("in 3 minutes (+3)");
                    })
                },
            )
            .unwrap(),
        ])
        .init();

        assert!(
            scheduler
                .upcoming_jobs
                .iter()
                .is_sorted_by(|a, b| a.1 <= b.1)
        );
        rt.block_on(async move {
            let mut scheduler = scheduler;

            let mut now = Zoned::now().datetime();
            let duration =
                every.get_next_time(now).duration_since(now).unsigned_abs();
            now += duration; // move to next minute

            let jobs = scheduler.__manual_tick(now);
            assert_eq!(jobs.len(), 2); // the * and +1 jobs should be here now
            join_all(jobs).await;
            assert!(
                scheduler
                    .upcoming_jobs
                    .iter()
                    .is_sorted_by(|a, b| a.1 <= b.1)
            );

            let duration =
                every.get_next_time(now).duration_since(now).unsigned_abs();
            now += duration; // move to next minute

            let jobs = scheduler.__manual_tick(now);
            assert_eq!(jobs.len(), 1); // the * job should be here now
            join_all(jobs).await;
            assert!(
                scheduler
                    .upcoming_jobs
                    .iter()
                    .is_sorted_by(|a, b| a.1 <= b.1)
            );

            let duration =
                every.get_next_time(now).duration_since(now).unsigned_abs();
            now += duration; // move to next minute

            let jobs = scheduler.__manual_tick(now);
            assert_eq!(jobs.len(), 2); // the * and +3 jobs should be here now
            join_all(jobs).await;
            assert!(
                scheduler
                    .upcoming_jobs
                    .iter()
                    .is_sorted_by(|a, b| a.1 <= b.1)
            );
        });
    }
}
