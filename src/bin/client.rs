use std::option::Option::Some;
use rshell::{CommandReader};
use std::path::{Path, PathBuf};
use std::env::{current_dir, set_current_dir};
use std::process::exit;

#[allow(unused_imports)]
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("用法: {} host port", args.get(0).unwrap());
        exit(0);
    }
    // "127.0.0.1", 9000
    for mut remote in rshell::shell(args.get(1).unwrap(), args.get(2).unwrap().parse().unwrap()) {
        while remote.is_live() {
            if let Some(mut command) = remote.read_command() {
                if command.trim_start().starts_with("cd ") {
                    command = command.replace("\n", "");
                    let mut result = format!("{}", &command);
                    let path = Path::new(&command.trim_start()[3..]);
                    let cd = match current_dir() {
                        Ok(dir) => {
                            if path.is_relative() {
                                dir.join(path)
                            } else {
                                PathBuf::from(path)
                            }
                        }
                        Err(_e) => {
                            PathBuf::from(path)
                        }
                    };
                    if let Err(e) = set_current_dir(cd) {
                        result = format!("没有那样的文件或目录: {}", e.to_string())
                    };
                    remote.write_result(Vec::from(result));
                } else if command.eq("exit") {
                    remote.exit();
                    drop(remote);
                    exit(0);
                } else {
                    match rshell::spawn(move|| {
                        match rshell::build_command().arg(&command).output() {
                            Ok(out) => {
                                if !out.stdout.is_empty() {
                                    out.stdout
                                } else if !out.stderr.is_empty() {
                                    out.stderr
                                } else {
                                    Vec::from(format!("命令已执行, 但获取输出失败({}): {}", match out.status.code() {
                                        None => { -1 }
                                        Some(n) => { n }
                                    }, &command))
                                }
                            }
                            Err(e) => {
                                Vec::from(e.to_string())
                            }
                        }
                    }, Duration::from_secs(60 * 5)) {
                        Ok(ret) => {
                            remote.write_result(ret);
                        }
                        Err(ret) => {
                            remote.write_result(Vec::from(ret.to_string()));
                        }
                    };
                }
            }
        }
    }
}
