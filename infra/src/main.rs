use anyhow::Result;
use std::{env, rc::Rc};
use walk_base_2_app::cui::StdCui;

fn main() -> Result<()> {
    let cui = Rc::new(StdCui {});
    walk_base_2::run(env::args(), cui)
}
