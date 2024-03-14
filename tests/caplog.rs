use caplog::{CapLog, CapRecord};
use log::{debug, info, Level};

#[test]
fn test_caplog_find() {
    let caplog = CapLog::new();

    let line = line!();
    info!("This is a log");
    debug!("Ignore me");

    let res = caplog.find(|r| r.level == Level::Info);

    assert_eq!(
        res,
        vec![CapRecord {
            level: Level::Info,
            target: "caplog".to_string(),
            msg: "This is a log".to_string(),
            module_path: Some(module_path!().to_string()),
            file: Some(file!().to_string()),
            line: Some(line + 1),
        }]
    );
}

#[test]
fn test_caplog_get_all() {
    let caplog = CapLog::new();

    let line = line!();
    info!("This is a log");
    debug!("This is another log");

    let res = caplog.get_all();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0].msg, "This is a log");
    assert_eq!(res[1].msg, "This is another log");

    assert_eq!(
        res,
        vec![
            CapRecord {
                level: Level::Info,
                target: "caplog".to_string(),
                msg: "This is a log".to_string(),
                module_path: Some(module_path!().to_string()),
                file: Some(file!().to_string()),
                line: Some(line + 1),
            },
            CapRecord {
                level: Level::Debug,
                target: "caplog".to_string(),
                msg: "This is another log".to_string(),
                module_path: Some(module_path!().to_string()),
                file: Some(file!().to_string()),
                line: Some(line + 2),
            },
        ]
    );
}
