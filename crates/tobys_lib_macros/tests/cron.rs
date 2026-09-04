#![allow(
    clippy::pedantic,
    clippy::tests_outside_test_module,
    clippy::panic_in_result_fn
)]

#[tokio::test]
#[cfg(feature = "cron")]
async fn test_cron() -> Result<(), ()> {
    use std::time::Duration;

    use futures::future::join_all;
    use tobys_lib::{
        create_cron_jobs, create_cron_time,
        cron::{CronTime, Scheduler},
    };
    use tokio::time::sleep;

    let every_minute = create_cron_time!(* * * * *);
    if every_minute != CronTime::new("* * * * *")? {
        return Err(());
    }

    let mut scheduler = Scheduler::new_with_jobs(create_cron_jobs!(
        * * * * * move || {
            Box::pin(async {
                println!("test 1!");
            })
        };
        * * * * * move || {
            Box::pin(async {
                println!("test 2!");
            })
        }
    ))
    .init();

    sleep(Duration::from_secs(60)).await;

    let jobs = scheduler.manual_tick();
    assert_eq!(jobs.len(), 2);
    join_all(jobs).await;

    Ok(())
}
