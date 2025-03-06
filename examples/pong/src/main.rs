use pong::start;

fn main() {
    async_std::task::block_on(start());
}
