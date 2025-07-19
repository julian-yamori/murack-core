use anyhow::Result;
use std::{env};
use walk_base_2_app::cui::StdCui;

#[tokio::main]
async fn main() -> Result<()> {
    walk_base_2::run(env::args(), StdCui {}).await
}
