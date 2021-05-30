use std::option::Option::Some;
use rshell::{CommandReader};
use std::path::{Path, PathBuf};
use std::env::{current_dir, set_current_dir};
use std::process::exit;

fn main() {
    let args:Vec<String> = std::env::args().collect();
    for mut remote in rshell::shell(args.get(1).unwrap(), args.get(2).unwrap().parse().unwrap()){
        while remote.is_live() {
            if  let Some(mut command) = remote.read_command(){
                if command.trim_start().starts_with("cd ") {
                    command = command.replace("\n", "");
                    let mut result = format!("{}", &command);
                    let path = Path::new(&command.trim_start()[3..]);
                    let cd = match current_dir() {
                        Ok(dir) => {
                            if path.is_relative() {
                                dir.join(path)
                            }else{
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

                }else if command.eq("exit") {
                    remote.exit();
                    drop(remote);
                    exit(0);
                } else{
                    match rshell::build_command().arg(command).output() {
                        Ok(out) => {
                            remote.write_result(out.stdout);
                            remote.write_result(out.stderr);
                        }
                        Err(e) => {
                            remote.write_result(Vec::from(e.to_string()));
                        }
                    };
                }
            }
        }
    }
}
