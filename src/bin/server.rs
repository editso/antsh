use rshell::{CommandWrite, CommandReader};
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("用法: {} host port", args.get(0).unwrap());
        exit(0)
    }
    for rsh in rshell::listen("0.0.0.0", args.get(1).unwrap().parse().unwrap()) {
        for mut remote in rsh {
            while remote.is_live() {
                if let Some(command) = remote.read_command() {
                    if let Some(result) = remote.write_command(command) {
                        println!("{}", result)
                    } else {
                        println!("error")
                    }
                }
            }
        }
    }
}