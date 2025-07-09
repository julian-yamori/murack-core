use crate::{AppArgs, AppComponentsImpl};
use anyhow::{Context, Result};
use std::{
    env,
    path::{Path, PathBuf},
    rc::Rc,
};
use walk_base_2_app::{Config, command::*, cui::Cui};

/// WalkBase2ライブラリのエントリポイント
/// # Arguments
/// - args: コマンドライン引数
pub fn run(args: impl Iterator<Item = String>, cui: Rc<dyn Cui>) -> Result<()> {
    let app_args = AppArgs::parse(args);
    let config = load_config(app_args.config_path.as_deref())?;

    let app_components = AppComponentsImpl::new(cui, config);

    match app_args.sub_command {
        //サブコマンドにより、コマンドオブジェクトを分岐
        Some(sub_command) => match &*sub_command {
            "add" => {
                CommandAdd::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            "check" => {
                CommandCheck::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            "move" => {
                CommandMove::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            "remove" => {
                CommandRemove::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            "playlist" => {
                CommandPlaylist::new(&app_components).run()?;
            }
            "replace" => {
                //todo app側で無効化中
                //CommandReplace::new(&app_args.sub_args[..], &app_components)?.run()?;
                todo!("not implemented");
            }
            "aw-get" => {
                CommandArtworkGet::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            "help" => {
                CommandHelp::new(&app_args.sub_args[..], &app_components)?.run()?;
            }
            _ => {
                return Err(walk_base_2_app::Error::InvalidCommandArgument {
                    msg: format!("sub command '{}' is invalid.", sub_command),
                }
                .into());
            }
        },
        //サブコマンドが空欄ならヘルプ出力
        None => {
            CommandHelp::new(&app_args.sub_args[..], &app_components)?.run()?;
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
