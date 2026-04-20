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

impl<N: Handler<LogRequest>> Handler<LogRequest> for InfoHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Info {
            println!("[INFO] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

#[handler]
struct WarningHandler<T> {}

impl<N: Handler<LogRequest>> Handler<LogRequest> for WarningHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Warning {
            println!("[WARN] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

#[handler]
struct ErrorHandler<T> {}

impl<N: Handler<LogRequest>> Handler<LogRequest> for ErrorHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Error {
            println!("[ERROR] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

fn main() {
    let logger = chain![
        |next| InfoHandler::new(next),
        |next| WarningHandler::new(next),
        |next| ErrorHandler::new(next),
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
