use std::{str::FromStr, time::Duration};

use chrono::{FixedOffset, Local};
use cron::Schedule;

// This function will run a cronjob every hour.
pub async fn hour_job() {
    /*
                  sec   min   hour   day of month   month   day of week   year
                  *     *     *      *              *       *             *
    */
    let expression = "0     0     *      *              *       *            ";
    let schedule = Schedule::from_str(expression).unwrap();
    let offset = Some(FixedOffset::east_opt(1 * 3600)).unwrap();

    loop {
        let mut upcoming = schedule
            .upcoming(offset.expect("Incorrect offset given."))
            .take(1);

        // Allows the terminal to not be blocked for closing with command + c
        std::thread::sleep(Duration::from_millis(500));

        let local = &Local::now();

        if let Some(datetime) = upcoming.next() {
            if datetime.timestamp() <= local.timestamp() {
                // Cron jobs go here.
            }
        }
    }
}
