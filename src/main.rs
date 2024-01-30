use gdke::spawn_and_inject;

fn main() {
    unsafe {
        spawn_and_inject("./z421.exe");
    }
    println!("hi");
}
