#![expect(clippy::tests_outside_test_module)]

#[tokio::test]
#[cfg(feature = "cron")]
async fn test_cron() {
    use tobys_lib::{create_cron_jobs, create_cron_time, cron::CronTime};

    let every_minute = create_cron_time!(* * * * *);
    assert_eq!(Ok(every_minute), CronTime::new("* * * * *"));

    assert_eq!(
        create_cron_jobs!(
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
        )
        .len(),
        2
    );
}
