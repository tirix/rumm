use crate::lang::database::Db;
use core::fmt::Formatter;
use core::fmt::Write;

pub trait Display {
    fn format(&self, f: &mut Formatter, db: &Db) -> std::result::Result<(), std::fmt::Error>;

    fn to_string(&self, db: &Db) -> String
    where
        Self: Sized,
    {
        let mut buf = String::new();
        write!(buf, "{}", DisplayPair(self, db)).unwrap();
        buf
    }
}

pub struct DisplayPair<'a>(pub &'a dyn Display, pub &'a Db);

impl core::fmt::Display for DisplayPair<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        self.0.format(f, self.1)
    }
}
