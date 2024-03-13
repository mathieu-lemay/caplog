use caplog::CapLog;
use log::{debug, info, Level};

#[test]
fn test_caplog_find() {
    let caplog = CapLog::new();

    info!("This is a log");
    debug!("This is another log");

    let res = caplog.find(|r| r.level == Level::Info);

    assert_eq!(res.len(), 1);
    assert_eq!(res[0].msg, "This is a log");
}

#[test]
fn test_caplog_get_all() {
    let caplog = CapLog::new();

    info!("This is a log");
    debug!("This is another log");

    let res = caplog.get_all();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0].msg, "This is a log");
    assert_eq!(res[1].msg, "This is another log");
}
