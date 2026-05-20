use crate::{Interval, interval_norm::IntervalNorm};
use chrono::Duration;

const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = SECS_PER_MINUTE * 60;
const SECS_PER_DAY: u64 = SECS_PER_HOUR * 24;
const NANOS_PER_SEC: i64 = 1_000_000_000;
const NANOS_PER_MICRO: i64 = 1000;

impl Interval {
    /// Tries to convert from the chrono `Duration` type to a `Interval`. Will
    /// return `None` on a overflow. This is a lossy conversion in that
    /// any units smaller than a microsecond will be lost.
    pub fn from_duration(duration: Duration) -> Option<Interval> {
        let days = duration.num_days();
        let mut new_dur = duration - Duration::days(days);
        let hours = new_dur.num_hours();
        new_dur -= Duration::hours(hours);
        let minutes = new_dur.num_minutes();
        new_dur -= Duration::minutes(minutes);
        let nano_secs = new_dur.num_nanoseconds()?;
        interval_from_parts(days, hours, minutes, nano_secs)
    }

    /// Tries to convert from the standard library `Duration` type to a
    /// `Interval`. Will return `None` on a overflow. This is a lossy conversion
    /// in that any units smaller than a microsecond will be lost.
    pub fn from_std_duration(duration: std::time::Duration) -> Option<Interval> {
        let total_secs = duration.as_secs();
        let days = i64::try_from(total_secs / SECS_PER_DAY).ok()?;
        let remaining_secs = total_secs % SECS_PER_DAY;
        let hours = i64::try_from(remaining_secs / SECS_PER_HOUR).ok()?;
        let remaining_secs = remaining_secs % SECS_PER_HOUR;
        let minutes = i64::try_from(remaining_secs / SECS_PER_MINUTE).ok()?;
        let seconds = i64::try_from(remaining_secs % SECS_PER_MINUTE).ok()?;
        let nano_secs = seconds
            .checked_mul(NANOS_PER_SEC)?
            .checked_add(i64::from(duration.subsec_nanos()))?;

        interval_from_parts(days, hours, minutes, nano_secs)
    }
}

fn interval_from_parts(
    mut days: i64,
    mut hours: i64,
    minutes: i64,
    nano_secs: i64,
) -> Option<Interval> {
    if days > (i32::MAX as i64) {
        let overflow_days = days - (i32::MAX as i64);
        let added_hours = overflow_days.checked_mul(24)?;
        hours = hours.checked_add(added_hours)?;
        days -= overflow_days;
    }
    let (seconds, remaining_nano) = reduce_by_units(nano_secs, NANOS_PER_SEC);
    // We have to discard any remaining nanoseconds
    let (microseconds, _remaining_nano) = reduce_by_units(remaining_nano, NANOS_PER_MICRO);
    let norm_interval = IntervalNorm {
        years: 0,
        months: 0,
        days: days as i32,
        hours,
        minutes,
        seconds,
        microseconds,
    };
    norm_interval.try_into_interval().ok()
}

fn reduce_by_units(nano_secs: i64, unit: i64) -> (i64, i64) {
    let new_time_unit = (nano_secs - (nano_secs % unit)) / unit;
    let remaining_nano = nano_secs - (new_time_unit * unit);
    (new_time_unit, remaining_nano)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::time::Duration as StdDuration;

    #[test]
    fn can_convert_small_amount_of_days() {
        let dur = Duration::days(5);
        let interval = Interval::from_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 5, 0)))
    }

    #[test]
    fn overflow_on_days() {
        let dur = Duration::days(100000000000);
        let interval = Interval::from_duration(dur);
        assert_eq!(interval, None)
    }

    #[test]
    fn can_convert_small_amount_of_secs() {
        let dur = Duration::seconds(1);
        let interval = Interval::from_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 0, 1_000_000)))
    }

    #[test]
    fn can_convert_one_micro() {
        let dur = Duration::nanoseconds(1000);
        let interval = Interval::from_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 0, 1)))
    }

    #[test]
    fn can_convert_std_duration_days() {
        let dur = StdDuration::from_secs(5 * SECS_PER_DAY);
        let interval = Interval::from_std_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 5, 0)))
    }

    #[test]
    fn can_convert_std_duration_secs() {
        let dur = StdDuration::from_secs(1);
        let interval = Interval::from_std_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 0, 1_000_000)))
    }

    #[test]
    fn can_convert_std_duration_one_micro() {
        let dur = StdDuration::from_nanos(1000);
        let interval = Interval::from_std_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 0, 1)))
    }

    #[test]
    fn std_duration_discards_submicrosecond_units() {
        let dur = StdDuration::from_nanos(999);
        let interval = Interval::from_std_duration(dur);
        assert_eq!(interval, Some(Interval::new(0, 0, 0)))
    }

    #[test]
    fn std_duration_overflows_when_interval_cannot_represent_it() {
        let interval = Interval::from_std_duration(StdDuration::from_secs(u64::MAX));
        assert_eq!(interval, None)
    }
}
