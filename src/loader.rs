use std::fs;

pub fn load_baud(path:&str)->Vec<u32>{
    match fs::read_to_string(path) {
        Ok(contents) => {
            let mut baud_list:Vec<u32> = vec![];
            for line in contents.lines(){
                baud_list.push(line.parse::<u32>().unwrap());
            }
            return baud_list;
        }
        Err(_) => {
            return vec![9600, 14400, 19200, 38400, 56000, 57600, 115200, 128000, 230400, 256000, 460800, 512000, 750000, 921600, 1500000];
        }
    } 
}