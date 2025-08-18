use murack_core_domain::{NonEmptyString, playlist::PlaylistTreeValue};
use sqlx::PgTransaction;

/// playlist コマンド用のプレイリストデータモデル
#[derive(Debug, PartialEq, Eq)]
pub struct CommandPlaylistModel {
    /// プレイリストID
    pub id: i32,

    /// プレイリスト名
    pub name: NonEmptyString,

    /// 親プレイリストID
    pub parent_id: Option<i32>,

    /// 親プレイリスト内でのインデックス
    pub in_folder_order: u32,

    /// DAPにこのプレイリストを保存するか
    pub save_dap: bool,
}

impl CommandPlaylistModel {
    pub async fn get_all_from_db<'c>(tx: &mut PgTransaction<'c>) -> anyhow::Result<Vec<Self>> {
        let playlists = sqlx::query!(
            r#"SELECT id, name AS "name: NonEmptyString", parent_id, in_folder_order, save_dap FROM playlists"#
        )
        .map(|record| Ok(CommandPlaylistModel {
            id: record.id,
            name: record.name,
            parent_id: record.parent_id,
            in_folder_order: record.in_folder_order.try_into()?,
            save_dap: record.save_dap,
        }))
        .fetch_all(&mut **tx)
        .await?
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(playlists)
    }
}

impl PlaylistTreeValue for CommandPlaylistModel {
    fn id(&self) -> i32 {
        self.id
    }

    fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    fn in_folder_order(&self) -> u32 {
        self.in_folder_order
    }
}
