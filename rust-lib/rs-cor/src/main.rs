use cor::{Handler, chain, handler};

#[derive(Clone)]
struct LogRequest {
    level: LogLevel,
    message: String,
}

#[derive(Clone, PartialEq)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

#[handler]
struct InfoHandler<T> {}

#[handler]
struct WarningHandler<T> {}

#[handler]
struct ErrorHandler<T> {}

fn main() {
    let logger = chain![
        |next| InfoHandler::new(
            |req: &LogRequest| req.level == LogLevel::Info,
            |req| println!("[INFO] {}", req.message),
            next,
        ),
        |next| WarningHandler::new(
            |req: &LogRequest| req.level == LogLevel::Warning,
            |req| println!("[WARN] {}", req.message),
            next,
        ),
        |next| ErrorHandler::new(
            |req: &LogRequest| req.level == LogLevel::Error,
            |req| println!("[ERROR] {}", req.message),
            next,
        ),
    ];

    logger.handle(LogRequest {
        level: LogLevel::Info,
        message: "Server started on port 8080".into(),
    });

    logger.handle(LogRequest {
        level: LogLevel::Warning,
        message: "Memory usage above 80%".into(),
    });

    logger.handle(LogRequest {
        level: LogLevel::Error,
        message: "Database connection lost".into(),
    });
}
