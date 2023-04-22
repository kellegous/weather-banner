use chrono::prelude::*;
use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Year {
    start: NaiveDate,
}

impl Year {
    pub fn from_start(start: NaiveDate) -> Year {
        Year { start }
    }

    pub fn from_ordinal(ord: i32) -> Year {
        Year {
            start: NaiveDate::from_yo_opt(ord, 1).unwrap(),
        }
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        NaiveDate::from_yo_opt(self.start.year() + 1, 1).unwrap()
    }

    pub fn duration(&self) -> Duration {
        self.end().signed_duration_since(self.start)
    }

    pub fn next(&self) -> Year {
        Self::from_ordinal(self.start.year() + 1)
    }

    pub fn days(&self) -> DaysIter {
        DaysIter {
            cur: Day::new(self.start),
            end: Day::new(self.end()),
        }
    }

    pub fn months(&self) -> MonthsIter {
        MonthsIter {
            cur: Month::from_start(self.start),
            end: Month::from_start(self.end()),
        }
    }

    pub fn ordinal(&self) -> i32 {
        self.start.year()
    }
}

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.start.year())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Month {
    start: NaiveDate,
}

impl Month {
    pub fn from_start(start: NaiveDate) -> Month {
        Month { start }
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        let start = self.start;
        NaiveDate::from_ymd_opt(
            match start.month() {
                12 => start.year() + 1,
                _ => start.year(),
            },
            match start.month() {
                12 => 1,
                _ => start.month() + 1,
            },
            1,
        )
        .unwrap()
    }

    pub fn duration(&self) -> Duration {
        self.end().signed_duration_since(self.start)
    }

    pub fn year(&self) -> Year {
        Year::from_ordinal(self.start.year())
    }

    pub fn next(&self) -> Month {
        Month::from_start(self.end())
    }

    pub fn days(&self) -> DaysIter {
        DaysIter {
            cur: Day::new(self.start),
            end: Day::new(self.end()),
        }
    }
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.start)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Day {
    t: NaiveDate,
}

impl Day {
    pub fn new(t: NaiveDate) -> Day {
        Day { t }
    }

    pub fn date(&self) -> NaiveDate {
        self.t
    }

    pub fn ordinal(&self) -> u32 {
        self.t.ordinal()
    }

    pub fn month(&self) -> Month {
        Month::from_start(NaiveDate::from_ymd_opt(self.t.year(), self.t.month(), 1).unwrap())
    }

    pub fn year(&self) -> Year {
        Year::from_ordinal(self.t.year())
    }

    pub fn next(&self) -> Day {
        Day::new(self.t + Duration::days(1))
    }

    pub fn prev(&self) -> Day {
        Day::new(self.t - Duration::days(1))
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.t)
    }
}

pub struct DaysIter {
    cur: Day,
    end: Day,
}

impl Iterator for DaysIter {
    type Item = Day;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        if cur.t != self.end.t {
            self.cur = cur.next();
            Some(cur)
        } else {
            None
        }
    }
}

pub struct MonthsIter {
    cur: Month,
    end: Month,
}

impl Iterator for MonthsIter {
    type Item = Month;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;
        if cur.start != self.end.start {
            self.cur = cur.next();
            Some(cur)
        } else {
            None
        }
    }
}
