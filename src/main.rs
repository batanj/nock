use nock::runtime;

fn main() {
    runtime::run::run().expect("event loop failed");
}
