use gen_tests::write_guest_test1;

fn main() {
    write_guest_test1("target/guest_test1.rs").expect("failed!");
}
