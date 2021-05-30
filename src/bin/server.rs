use rshell::{CommandWrite, CommandReader};

fn main() {

    for rsh in rshell::listen("0.0.0.0", 9000) {
        for mut remote in rsh {
            while remote.is_live() {
               if let Some(command) = remote.read_command(){
                    if let Some(result) = remote.write_command(command){
                        println!("{}",  result)
                    }else{
                        println!("error")
                    }
               }
            }
        }
    };
}