use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;
use std::sync::Once;

static LOGGER: TestLogger = TestLogger;
static LOG_INIT_ONCE: Once = Once::new();
thread_local!(static LOG_RECORDS: RefCell<Vec<CapRecord>> = RefCell::new(Vec::new()));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapRecord {
    pub level: Level,
    pub target: String,
    pub msg: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl From<&Record<'_>> for CapRecord {
    fn from(r: &Record) -> Self {
        Self {
            level: r.metadata().level(),
            target: r.metadata().target().to_owned(),
            msg: r.args().to_string(),
            module_path: r.module_path().map(|s| s.to_string()),
            file: r.file().map(|s| s.to_string()),
            line: r.line(),
        }
    }
}

struct TestLogger;

impl Log for TestLogger {
    /// This logger is always enabled, in order to ensure we record everything.
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        LOG_RECORDS.with(|records| records.borrow_mut().push(record.into()))
    }

    fn flush(&self) {}
}

pub struct CapLog {}

impl CapLog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        LOG_INIT_ONCE.call_once(|| {
            log::set_logger(&LOGGER)
                .map(|()| log::set_max_level(log::LevelFilter::Trace))
                .expect("Error initializing test logger")
        });

        Self {}
    }

    pub fn get_all(&self) -> Vec<CapRecord> {
        LOG_RECORDS.with(|records| records.borrow().iter().cloned().collect())
    }

    pub fn find<F>(&self, matcher: F) -> Vec<CapRecord>
    where
        F: Fn(&CapRecord) -> bool,
    {
        LOG_RECORDS.with(|records| {
            records
                .borrow()
                .iter()
                .filter(|r| matcher(r))
                .cloned()
                .collect()
        })
    }

    pub fn clear(&mut self) {
        LOG_RECORDS.with(|records| records.borrow_mut().clear())
    }
}

impl Drop for CapLog {
    fn drop(&mut self) {
        self.clear()
    }
}

#[cfg(test)]
mod test_caplog {
    use super::{CapLog, CapRecord, LOG_RECORDS};

    use log::{debug, error, info, trace, warn, Level};
    use std::{thread, time::Duration};

    #[test]
    fn test_logs_are_cleared_when_caplog_goes_out_of_scope() {
        {
            let _c = CapLog::new();

            info!("foobar");
            assert_eq!(LOG_RECORDS.with(|records| (records.borrow()).len()), 1);

            info!("baz");
            assert_eq!(LOG_RECORDS.with(|records| (records.borrow()).len()), 2);
        }

        assert_eq!(LOG_RECORDS.with(|records| (records.borrow()).len()), 0);
    }

    #[test]
    fn test_captured_logs_are_not_shared_between_threads() {
        for _ in 0..16 {
            thread::spawn(|| {
                let _c = CapLog::new();

                info!("foobar");
                assert_eq!(LOG_RECORDS.with(|records| (records.borrow()).len()), 1);

                thread::sleep(Duration::from_millis(5));

                info!("baz");
                assert_eq!(LOG_RECORDS.with(|records| (records.borrow()).len()), 2);
            });
        }
    }

    #[test]
    fn test_message_contains_the_formatted_message() {
        let c = CapLog::new();

        info!("{} + {} = {:.3}", 0.1, 0.2, 0.1 + 0.2);

        assert_eq!(c.find(|_| true)[0].msg, "0.1 + 0.2 = 0.300");
    }

    #[test]
    fn test_captured_log_contains_all_relevant_metadata() {
        let c = CapLog::new();

        info!(target: "target", "test");
        let line = line!() - 1;

        let record = &c.find(|_| true)[0];

        assert_eq!(
            record,
            &CapRecord {
                level: Level::Info,
                target: "target".to_string(),
                msg: "test".to_string(),
                line: Some(line),
                module_path: Some(module_path!().to_string()),
                file: Some(file!().to_string()),
            }
        )
    }

    #[test]
    fn test_all_levels_are_captured() {
        let c = CapLog::new();

        trace!("foo");
        debug!("foo");
        info!("foo");
        warn!("foo");
        error!("foo");

        for lvl in [
            Level::Trace,
            Level::Debug,
            Level::Info,
            Level::Warn,
            Level::Error,
        ] {
            assert_eq!(c.find(|r| r.level == lvl).len(), 1);
        }
    }

    #[test]
    fn test_find_logs_returns_matching_records() {
        let c = CapLog::new();

        info!("foobar");
        error!("baz");

        assert_eq!(c.find(|_| true).len(), 2);
        assert_eq!(c.find(|r| r.level == Level::Debug).len(), 0);

        let res = c.find(|r| r.msg == "foobar");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].level, Level::Info);
        assert_eq!(res[0].msg, "foobar");

        let res = c.find(|r| r.level == Level::Error);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].level, Level::Error);
        assert_eq!(res[0].msg, "baz");

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].level, Level::Error);
        assert_eq!(res[0].msg, "baz");
    }

    #[test]
    fn test_get_all_returns_all_records() {
        let c = CapLog::new();

        info!("foobar");
        error!("baz");

        let res = c.get_all();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0].msg, "foobar");
        assert_eq!(res[1].msg, "baz");
    }

    #[test]
    fn test_clear_resets_the_captured_logs() {
        let mut c = CapLog::new();

        info!("foobar");
        error!("baz");

        assert_eq!(c.find(|_| true).len(), 2);

        c.clear();
        assert_eq!(c.find(|_| true).len(), 0);

        info!("foobar");
        assert_eq!(c.find(|_| true).len(), 1);
    }
}
