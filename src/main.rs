use poggers::{external::process::ExProcess, traits::Mem};

fn main() {
    let mut pid = 0;
    {
        let proc = ExProcess::new_from_name("4 Test.exe".to_string()).unwrap();
        pid = proc.get_pid();
    }

    let key = gdke::get_from_pid(pid).expect("unable to find key");
    
    println!("Key: {}", key);
}
