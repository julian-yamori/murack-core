//! 旧 track_usecase の関数

use anyhow::Result;
use murack_core_domain::{
    Error, NonEmptyString,
    folder::{FolderIdMayRoot, folder_repository},
    path::{LibraryDirectoryPath, LibraryTrackPath},
    playlist::playlist_repository,
    track::track_repository,
};
use sqlx::PgTransaction;

/// パス文字列を指定してDBの曲パスを移動
pub async fn move_path_str_db<'c>(
    tx: &mut PgTransaction<'c>,
    src: &NonEmptyString,
    dest: &NonEmptyString,
) -> Result<()> {
    // パス文字列がファイルかどうかを、完全一致するパスの曲が DB に存在するかどうかで判定
    let src_as_track: LibraryTrackPath = src.clone().into();
    let track_exists = track_repository::is_exist_path(tx, &src_as_track).await?;

    if track_exists {
        // 指定された 1 曲だけ処理

        let dest_as_track: LibraryTrackPath = dest.clone().into();

        move_track_db_unit(tx, &src_as_track, &dest_as_track).await?;
    } else {
        // 指定ディレクトリ以下の全ての曲について、パスの変更を反映

        let src_as_dir: LibraryDirectoryPath = src.clone().into();
        let dest_as_dir: LibraryDirectoryPath = dest.clone().into();

        for src_track in track_repository::get_path_by_directory(tx, &src_as_dir).await? {
            let dest_track = src_child_path_to_dest(&src_track, &src_as_dir, &dest_as_dir)?;
            move_track_db_unit(tx, &src_track, &dest_track).await?;
        }
    };

    Ok(())
}

/// 曲一つのDB内パス移動処理
async fn move_track_db_unit<'c>(
    tx: &mut PgTransaction<'c>,
    src: &LibraryTrackPath,
    dest: &LibraryTrackPath,
) -> Result<()> {
    if track_repository::is_exist_path(tx, dest).await? {
        return Err(Error::DbTrackAlreadyExists(dest.to_owned()).into());
    }

    //移動先の親フォルダを登録してIDを取得
    let dest_parent_opt = dest.parent();
    let new_folder_id = match dest_parent_opt {
        None => FolderIdMayRoot::Root,
        Some(dest_parent) => {
            let id = folder_repository::register_not_exists(tx, &dest_parent).await?;
            FolderIdMayRoot::Folder(id)
        }
    };

    //曲のパス情報を変更
    track_repository::update_path(tx, src, dest, new_folder_id).await?;

    //子要素がなくなった親フォルダを削除
    if let Some(parent) = src.parent() {
        folder_repository::delete_db_if_empty(tx, &parent).await?;
    }

    //パスを使用したフィルタがあるかもしれないので、
    //プレイリストのリストアップ済みフラグを解除
    playlist_repository::reset_listuped_flag(tx).await?;
    //プレイリストファイル内のパスだけ変わるので、
    //DAP変更フラグを立てる
    playlist_repository::set_dap_change_flag_all(tx, true).await?;

    Ok(())
}

/// move コマンドで指定された src_dir の子の src_track から、dest_dir の子として移動する先のパスを取得
fn src_child_path_to_dest(
    src_track: &LibraryTrackPath,
    src_dir: &LibraryDirectoryPath,
    dest_dir: &LibraryDirectoryPath,
) -> anyhow::Result<LibraryTrackPath> {
    let src_dir_str: &str = src_dir.as_ref();
    let src_track_str: &str = src_track.as_ref();

    // src_track が src_dir で始まっているか確認
    if !src_track_str.starts_with(src_dir_str) {
        return Err(Error::GetRelativePathFailed {
            track: src_track.to_owned(),
            parent: src_dir.to_owned(),
        }
        .into());
    }

    let relative_path = &src_track_str[src_dir_str.len()..];

    // 文字列を取得して連結
    let mut s = (dest_dir.as_ref() as &NonEmptyString).clone();
    s.push_str(relative_path);

    Ok(s.into())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use murack_core_domain::path::LibraryDirectoryPath;
    use test_case::test_case;

    use super::*;

    #[test_case("parent/src/dir/file.flac", "parent/src", "parent", "parent/dir/file.flac"; "1階層上に移動")]
    #[test_case("parent/dir/file.flac", "parent", "parent/dest", "parent/dest/dir/file.flac"; "親ディレクトリの1階層下に移動")]
    #[test_case("src/dir/file.flac", "src", "parent/dest", "parent/dest/dir/file.flac"; "別ディレクトリの1階層下に移動")]
    #[test_case("parent/src/file.flac", "parent/src", "parent/dest", "parent/dest/file.flac"; "子ディレクトリなし")]
    #[test_case("src/dir/dir2/file.flac", "src", "dest", "dest/dir/dir2/file.flac"; "子ディレクトリが2階層")]
    fn test_src_child_path_to_dest(
        src_track: &str,
        src_dir: &str,
        dest_dir: &str,
        expect_dest_track: &str,
    ) -> anyhow::Result<()> {
        let actual = src_child_path_to_dest(
            &LibraryTrackPath::from_str(src_track)?,
            &LibraryDirectoryPath::from_str(src_dir)?,
            &LibraryDirectoryPath::from_str(dest_dir)?,
        )?;

        assert_eq!(actual.as_ref() as &str, expect_dest_track);
        Ok(())
    }
}
