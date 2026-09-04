fn main() -> Result<(), ()> {
    #[cfg(feature = "cron")]
    {
        use tobys_lib::{
            create_cron_jobs, create_cron_time,
            cron::{CronTime, Scheduler},
        };

        let every_minute = create_cron_time!(* * * * *);
        if every_minute != CronTime::new("* * * * *")? {
            return Err(());
        }

        let mut scheduler = Scheduler::new_with_jobs(create_cron_jobs!(
            * * * * *, move || {
                Box::pin(async {
                    println!("test 1!");
                })
            };
            * * * * *, move || {
                Box::pin(async {
                    println!("test 2!");
                })
            }
        ))
        .init();

        scheduler.manual_tick();

        Ok(())
    }

    #[cfg(not(feature = "cron"))]
    Ok(())
}
