use anyhow::Result;
use murack_core_app::cui::StdCui;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    walk_base_2::run(env::args(), StdCui {}).await
}
