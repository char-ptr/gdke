use gdke::spawn_and_inject;

fn main() {
    unsafe {
        let key = spawn_and_inject(
            &std::env::args().nth(1).unwrap(),
            &std::env::args().nth(2).unwrap(),
        )
        .expect("failed to resolve key");
        print!("key: ");
        for val in key {
            print!("{:0>2x}", val);
        }
        println!("\npress enter to close");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
