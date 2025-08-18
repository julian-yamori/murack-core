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
    pub in_folder_order: i32,

    /// DAPにこのプレイリストを保存するか
    pub save_dap: bool,
}

impl CommandPlaylistModel {
    pub async fn get_all_from_db<'c>(tx: &mut PgTransaction<'c>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(Self,
            r#"SELECT id, name AS "name: NonEmptyString", parent_id, in_folder_order, save_dap FROM playlists"#
        )
        .fetch_all(&mut **tx)
        .await
    }
}

impl PlaylistTreeValue for CommandPlaylistModel {
    fn id(&self) -> i32 {
        self.id
    }

    fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    fn in_folder_order(&self) -> i32 {
        self.in_folder_order
    }
}
