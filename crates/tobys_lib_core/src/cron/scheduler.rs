use ::core::{cmp::Ordering, ops::Deref};
use jiff::civil::DateTime;

use crate::{alias::Vec, cron::Job};

struct LockedJob<'a> {
    id: usize,
    job: Job<'a>,
}
impl<'a> Deref for LockedJob<'a> {
    type Target = Job<'a>;

    fn deref(&self) -> &Self::Target {
        &self.job
    }
}

struct JobOrdering<'a, 'b>(&'b LockedJob<'a>, DateTime);
impl PartialEq for JobOrdering<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for JobOrdering<'_, '_> {}
impl PartialOrd for JobOrdering<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JobOrdering<'_, '_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

/// The scheduler for cron jobs.
#[derive(Default)]
#[must_use = "The scheduler is lazy and does nothing on its own without using it"]
pub struct Scheduler<'a> {
    all_jobs: Vec<LockedJob<'a>>,
    upcoming_jobs: Vec<JobOrdering<'a, 'a>>,
    initialized: bool,
}

impl<'a> Scheduler<'a> {
    /// Creates a new [`Scheduler`] for cron [`Job`]s.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Scheduler`] and adds all [`Job`]s in the iterator.
    pub fn new_with_jobs<I: IntoIterator<Item = Job<'a>>>(jobs: I) -> Self {
        let all_jobs: Vec<_> = jobs
            .into_iter()
            .enumerate()
            .map(|(id, job)| LockedJob { id, job })
            .collect();
        let len = all_jobs.len();

        Self {
            all_jobs,
            upcoming_jobs: Vec::with_capacity(len),
            initialized: false,
        }
    }

    /// Add a [`Job`] to the [`Scheduler`].
    #[expect(clippy::missing_panics_doc, clippy::unwrap_used)]
    pub fn add_job(&'a mut self, job: Job<'a>) {
        self.all_jobs.push(LockedJob {
            id: self.all_jobs.len(),
            job,
        });
        if self.initialized {
            let last = self.all_jobs.last().unwrap();
            let job = JobOrdering(last, last.get_next_time());
            match self.upcoming_jobs.binary_search(&job) {
                Ok(index) | Err(index) => {
                    self.upcoming_jobs.insert(index, job);
                }
            }
        }
    }

    /// Initializes the scheduler.
    ///
    /// This function prepares the scheduler for running, and must be called
    pub fn init(&'a mut self) {
        let mut jobs_and_next_date: Vec<_> = self
            .all_jobs
            .iter()
            .map(|v| JobOrdering(v, v.get_next_time()))
            .collect();
        jobs_and_next_date.sort();

        self.upcoming_jobs.extend(jobs_and_next_date);
        self.initialized = true;
    }

    /// # Errors
    ///
    /// This will error if the [`Scheduler`] is uninitialized.
    pub async fn tick(&'a mut self) -> Result<(), ()> {
        if !self.initialized {
            return Err(());
        }

        let now = jiff::Zoned::now();

        let jobs_to_run =
            self.upcoming_jobs.extract_if(.., |v| v.1 <= now.datetime());
        for job in jobs_to_run {
            job.0.run_func().await;
        }

        Ok(())
    }

    /// Manual tick
    pub async fn manual_tick(&mut self) {}
}
