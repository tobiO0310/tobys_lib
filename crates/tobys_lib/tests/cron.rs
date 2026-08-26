fn main() -> Result<(), ()> {
    #[cfg(all(
        feature = "cron",
        any(feature = "alloc", feature = "std"),
        feature = "macros"
    ))]
    {
        use tobys_lib::{create_cron, create_jobs, cron::CronTime};

        let every_minute = create_cron!(* * * * *);
        if every_minute != CronTime::new("* * * * *")? {
            return Err(());
        }

        let jobs = create_jobs!(
            * * * * *, move || {
                Box::pin(async {
                    println!("test!");
                })
            };
            * * * * *, move || {
                Box::pin(async {
                    println!("test!");
                })
            }
        );

        println!("{every_minute}");

        Ok(())
    }
}
