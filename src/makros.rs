#[macro_export]
macro_rules! empty_exit {
    ($msg:expr) => {{
        use owo_colors::OwoColorize;
        println!("{}", format!("{}", $msg.yellow()));
        std::process::exit(0)
    }};
}

#[macro_export]
/// exit with error
macro_rules! err_exit {
    ($msg:expr) => {
        use owo_colors::OwoColorize;
        eprintln!("{}", format!("ERROR: {}", $msg).red());
        std::process::exit(1)
    };
}

#[macro_export]
macro_rules! suc_exit {
    ($msg:expr) => {{
        use owo_colors::OwoColorize;
        println!("{}", format!("{}: {}", "WARNING".yellow(), $msg));
        std::process::exit(0)
    }};
}

#[macro_export]
macro_rules! warn {
    ($msg:expr) => {{
        use owo_colors::OwoColorize;
        println!("{}", format!("{}: {}", "WARNING".yellow(), $msg));
    }};
}
