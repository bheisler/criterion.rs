//! Private macro used for error handling.

/// Logs an error, ignores an `Ok` value.
macro_rules! log_if_err {
    ($x:expr) => {
        let closure = || { try_else_return!($x); };
        closure();
    };
}

/// Matches a result, returning the `Ok` value in case of success,
/// exits the calling function otherwise.
/// A closure which returns the return value for the function can
/// be passed as second parameter.
macro_rules! try_else_return {
    ($x:expr) => {
        try_else_return!($x, || {});
    };
    ($x:expr, $el:expr) => {
        match $x {
            Ok(x) => x,
            Err(e) => {
                error!("error: {}", e.cause());
                for cause in e.causes() {
                    error!("caused by: {}", cause);
                }

                debug!("backtrace: {}", e.backtrace());

                let closure = $el;
                return closure();
            },
        }
    };
}
