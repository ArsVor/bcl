#[macro_export]
macro_rules! err_exit {
    ($msg:expr) => {
        println!("ERROR: {}", $msg);
        std::process::exit(1)
    };
}
