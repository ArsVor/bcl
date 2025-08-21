#[macro_export]
macro_rules! err_exit {
    ($msg:expr) => {
        use owo_colors::OwoColorize;
        println!("{}", format!("ERROR: {}", $msg).red());
        std::process::exit(1)
    };
}
