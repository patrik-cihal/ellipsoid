use simple_app_readme::start;

fn main() {
    async_std::task::block_on(start());
}
