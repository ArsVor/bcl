mod makros;
pub mod structs;

use std::env::args;

use structs::Command;

fn main() {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    // println!("{:?}", &args);
    if !args.is_empty() {
        let command: Command = Command::from(args);
        println!("{:?}", &command);
        println!("Is year? - {:?}", &command.date.year.is_some());
        println!("Year is - {:?}", &command.date.year_or_now());
        println!("Now year is - {:?}", &command.date.year);
        println!("Is date a valid? - {:?}", &command.date.is_valid_date());
        println!("{:?}", &command.funk.is_some());
        // let funk = command.fu
        println!("{:?}", &command.val.unwrap_or(0.0));
        println!("done")
    } else {
        err_exit!("Nothing to do (from main.rs)");
        // потім реалізую логіку виводу help
    }

}
