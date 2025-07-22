use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use murack_core_app::{Config, cui::Cui};

use crate::{AppArgs, Registry};

/// murack-core app のエントリポイント
/// # Arguments
/// - args: コマンドライン引数
pub async fn run(args: impl Iterator<Item = String>, cui: impl Cui + Sync + Send) -> Result<()> {
    let app_args = AppArgs::parse(args);
    let config = load_config(app_args.config_path.as_deref())?;

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&config.database_url)
        .await?;

    let registry = Registry::new(cui, config);

    match app_args.sub_command {
        //サブコマンドにより、コマンドオブジェクトを分岐
        Some(sub_command) => match &*sub_command {
            "add" => {
                registry
                    .command_add(&app_args.sub_args[..])?
                    .run(&db_pool)
                    .await?;
            }
            "check" => {
                registry
                    .command_check(&app_args.sub_args[..])?
                    .run(&db_pool)
                    .await?;
            }
            "move" => {
                registry
                    .command_move(&app_args.sub_args[..])?
                    .run(&db_pool)
                    .await?;
            }
            "remove" => {
                registry
                    .command_remove(&app_args.sub_args[..])?
                    .run(&db_pool)
                    .await?;
            }
            "playlist" => {
                registry.command_playlist().run(&db_pool).await?;
            }
            "replace" => {
                //todo app側で無効化中
                //CommandReplace::new(&app_args.sub_args[..], &app_components)?.run()?;
                todo!("not implemented");
            }
            "aw-get" => {
                registry
                    .command_artwork_get(&app_args.sub_args[..])?
                    .run()?;
            }
            "help" => {
                registry.command_help(&app_args.sub_args[..])?.run()?;
            }
            _ => {
                return Err(murack_core_app::Error::InvalidCommandArgument {
                    msg: format!("sub command '{sub_command}' is invalid."),
                }
                .into());
            }
        },
        //サブコマンドが空欄ならヘルプ出力
        None => {
            registry.command_help(&app_args.sub_args[..])?.run()?;
        }
    }

    Ok(())
}

/// 設定ファイルを読み込み
fn load_config(path: Option<&Path>) -> Result<Config> {
    match path {
        Some(p) => Config::load(p),
        None => {
            let p = get_config_path_default()?;
            Config::load(&p)
        }
    }
}

/// デフォルトの設定ファイルのパスを取得
fn get_config_path_default() -> Result<PathBuf> {
    let mut path = env::current_exe()
        .with_context(|| "実行ファイルのパスの取得に失敗しました。".to_owned())?;

    path.set_file_name("config.toml");

    Ok(path)
}
