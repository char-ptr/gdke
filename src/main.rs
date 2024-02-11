use gdke::spawn_and_inject;

fn main() {
    unsafe {
        spawn_and_inject(&std::env::args().nth(1).unwrap());
    }
}
