use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;
use murack_core_domain::{
    Error as DomainError,
    artwork::{TrackArtwork, artwork_repository},
    path::LibraryTrackPath,
    track::TrackItemKind,
};
use sqlx::PgPool;

use super::{TrackItemConflict, messages};
use crate::{
    Config,
    command::check::domain::check_usecase,
    cui::Cui,
    data_file,
    track_sync::{DbTrackSync, TrackSync, track_sync_repository},
};

/// データ内容同一性についての解決処理
#[automock]
#[async_trait]
pub trait ResolveDataMatch {
    /// データ内容同一性についての解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve(&self, db_pool: &PgPool, track_path: &LibraryTrackPath) -> Result<bool>;
}

/// ResolveDataMatchの実装
pub struct ResolveDataMatchImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    config: &'config Config,
    cui: &'cui CUI,
}

#[async_trait]
impl<'config, 'cui, CUI> ResolveDataMatch for ResolveDataMatchImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    /// データ内容同一性についての解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve(&self, db_pool: &PgPool, track_path: &LibraryTrackPath) -> Result<bool> {
        //データ読み込み
        let mut pc_data = data_file::read_track_sync(&self.config.pc_lib, track_path)?;
        let mut db_data = self.load_db_track(db_pool, track_path).await?;

        if !self
            .resolve_editable(db_pool, &mut pc_data, &mut db_data)
            .await?
        {
            return Ok(false);
        }

        if !self
            .resolve_artwork(db_pool, &mut pc_data, &mut db_data)
            .await?
        {
            return Ok(false);
        }

        if !self
            .resolve_duration(db_pool, &mut pc_data, &mut db_data)
            .await?
        {
            return Ok(false);
        }

        Ok(true)
    }
}

impl<'config, 'cui, CUI> ResolveDataMatchImpl<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    pub fn new(config: &'config Config, cui: &'cui CUI) -> Self {
        Self { config, cui }
    }

    /// PC・DB間の、曲情報(編集可能部)の齟齬の解決
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve_editable(
        &self,
        db_pool: &PgPool,
        pc_track: &mut TrackSync,
        db_track: &mut DbTrackSync,
    ) -> Result<bool> {
        //全体を比較して齟齬リストを取得
        let conflict_items = check_usecase::check_editable(pc_track, &db_track.track_sync);
        //齟齬がなければ次の処理へ
        if conflict_items.is_empty() {
            return Ok(true);
        }

        let conflicts: Vec<_> = conflict_items
            .into_iter()
            .map(|item_kind| TrackItemConflict { item_kind })
            .collect();

        let cui = &self.cui;

        //結果表示
        cui_outln!(cui, "----")?;
        self.display_all_conflicts(&conflicts, pc_track, &db_track.track_sync)?;

        cui_outln!(cui, "1: PCからDBへ上書き")?;
        cui_outln!(cui, "2: DBからPCへ上書きし、DAPも更新")?;
        cui_outln!(cui, "3: それぞれの項目個別に処理方法を選択")?;
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['1', '2', '3', '0', '-'], messages::MSG_SELECT_OPERATION)?;

        match input {
            //PCからDBへ上書き
            '1' => {
                self.overwrite_track_editable(pc_track, &mut db_track.track_sync);
                self.save_db_exclude_artwork(db_pool, db_track).await?;

                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                let track_path = &db_track.path;

                //PCのデータを上書き
                self.overwrite_track_editable(&db_track.track_sync, pc_track);
                self.overwrite_pc_track_file(track_path, pc_track)?;

                //DAPのデータをPCのデータで上書き
                data_file::overwrite_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;

                Ok(true)
            }
            //それぞれの項目個別に処理方法を選択
            '3' => {
                for conflict in conflicts {
                    if !self
                        .resolve_each_property(db_pool, pc_track, db_track, &conflict)
                        .await?
                    {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }

    /// 曲の値1つについての、PC・DB間の齟齬解決
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve_each_property(
        &self,
        db_pool: &PgPool,
        pc_track: &mut TrackSync,
        db_track: &mut DbTrackSync,
        conflict: &TrackItemConflict,
    ) -> Result<bool> {
        let db_sync = &mut db_track.track_sync;

        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", conflict.item_name())?;

            cui_outln!(cui, "[PC]")?;
            cui_outln!(
                cui,
                "{}",
                conflict
                    .display_value(pc_track)
                    .as_deref()
                    .unwrap_or("None")
            )?;

            cui_outln!(cui, "[DB]")?;
            cui_outln!(
                cui,
                "{}",
                conflict.display_value(db_sync).as_deref().unwrap_or("None")
            )?;

            cui_outln!(cui)?;

            cui_outln!(cui, "1: PCからDBへ上書き")?;
            cui_outln!(cui, "2: DBからPCへ上書きし、DAPも更新")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //PCからDBへ上書き
            '1' => {
                //DBの値を上書きする
                conflict.copy_each_sync(pc_track, db_sync);
                self.save_db_exclude_artwork(db_pool, db_track).await?;

                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                //PCの値を上書きする
                conflict.copy_each_sync(db_sync, pc_track);

                let track_path = &db_track.path;
                self.overwrite_pc_track_file(track_path, pc_track)?;

                //DAPのデータをPCのデータで上書き
                data_file::overwrite_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }

    /// PC・DB間のアートワークの齟齬の解決処理
    ///
    /// # Returns
    /// 次の曲の解決処理へ継続するか
    async fn resolve_artwork(
        &self,
        db_pool: &PgPool,
        pc_track: &mut TrackSync,
        db_track: &mut DbTrackSync,
    ) -> Result<bool> {
        //アートワークが一致したらスキップ
        if check_usecase::check_artwork(pc_track, &db_track.track_sync) {
            return Ok(true);
        }

        let cui = &self.cui;

        cui_outln!(cui, "----")?;
        //PCのアートワーク情報を表示
        cui_outln!(cui, "[PC]")?;
        self.display_artwork(&pc_track.artworks)?;
        cui_outln!(cui)?;

        //DBのアートワーク情報を表示
        cui_outln!(cui, "[DB]")?;
        self.display_artwork(&db_track.track_sync.artworks)?;
        cui_outln!(cui)?;

        cui_outln!(cui, "1: PCからDBへ上書き")?;
        cui_outln!(cui, "2: DBからPCへ上書きし、DAPも更新")?;
        cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
        cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
        cui_outln!(cui)?;

        let input = cui.input_case(&['1', '2', '0', '-'], messages::MSG_SELECT_OPERATION)?;

        match input {
            //PCからDBへ上書き
            '1' => {
                let mut tx = db_pool.begin().await?;

                //DBに上書き保存
                let track_id = db_track.id;

                artwork_repository::register_track_artworks(&mut tx, track_id, &pc_track.artworks)
                    .await?;

                //念の為、保存した値で変数値を上書きしておく
                db_track.track_sync.artworks =
                    artwork_repository::get_track_artworks(&mut tx, track_id).await?;

                tx.commit().await?;
                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                //PCのデータを上書き
                pc_track.artworks = db_track.track_sync.artworks.clone();

                //PCに保存
                let track_path = &db_track.path;
                self.overwrite_pc_track_file(track_path, pc_track)?;

                //DAPのデータをPCのデータで上書き
                data_file::overwrite_track_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    track_path,
                )?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }

    /// PC・DB間の、再生時間の齟齬の解決
    ///
    /// # Returns
    /// 次の曲の解決処理へ継続するか
    async fn resolve_duration(
        &self,
        db_pool: &PgPool,
        pc_track: &mut TrackSync,
        db_track: &mut DbTrackSync,
    ) -> Result<bool> {
        //再生時間が一致したらスキップ
        if check_usecase::check_duration(pc_track, &db_track.track_sync) {
            return Ok(true);
        }

        let input = {
            let cui = &self.cui;

            //再生時間の齟齬を表示
            cui_outln!(cui, "----")?;
            cui_outln!(
                cui,
                "* 再生時間: {}ms | {}ms",
                pc_track.duration,
                db_track.track_sync.duration
            )?;
            cui_outln!(cui, "PC vs DB")?;
            cui_outln!(cui)?;

            cui_outln!(cui, "1: PCからDBへ上書き")?;
            cui_outln!(cui, "{}", messages::CASE_MSG_DONT_RESOLVE)?;
            cui_outln!(cui, "{}", messages::CASE_MSG_TERMINATE)?;
            cui_outln!(cui)?;

            cui.input_case(&['1', '0', '-'], messages::MSG_SELECT_OPERATION)?
        };

        match input {
            //PCからDBへ上書き
            '1' => {
                //DB側の再生時間を上書きして保存
                db_track.track_sync.duration = pc_track.duration;
                self.save_db_exclude_artwork(db_pool, db_track).await?;

                // 再生時間だけ書き換える方がいいかも……？
                // sqlx::query!(
                //     "UPDATE tracks SET duration = $1 WHERE id = $2",
                //     i32::try_from(duration)?,
                //     track_id,
                // )
                // .execute(&mut **tx)
                // .await?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }

    /// DBから曲ファイルを読み込み
    async fn load_db_track(
        &self,
        db_pool: &PgPool,
        track_path: &LibraryTrackPath,
    ) -> Result<DbTrackSync> {
        let mut tx = db_pool.begin().await?;

        track_sync_repository::get_by_path(&mut tx, track_path)
            .await?
            .ok_or_else(|| DomainError::DbTrackNotFound(track_path.clone()).into())
    }

    /// DBに曲の連携情報(アートワーク以外)を保存
    async fn save_db_exclude_artwork(
        &self,
        db_pool: &PgPool,
        db_track: &DbTrackSync,
    ) -> Result<()> {
        let mut tx = db_pool.begin().await?;

        track_sync_repository::save_exclude_artwork(&mut tx, db_track).await?;

        tx.commit().await?;
        Ok(())
    }

    /// PCのファイルの曲データを上書き
    fn overwrite_pc_track_file(
        &self,
        track_path: &LibraryTrackPath,
        pc_track: &TrackSync,
    ) -> Result<()> {
        data_file::overwrite_track_sync(&self.config.pc_lib, track_path, pc_track)?;

        Ok(())
    }

    /// 曲の編集可能データを上書き
    ///
    /// # todo
    /// domainに移動
    fn overwrite_track_editable(&self, src_track: &TrackSync, dest_track: &mut TrackSync) {
        dest_track.title = src_track.title.clone();
        dest_track.artist = src_track.artist.clone();
        dest_track.album = src_track.album.clone();
        dest_track.genre = src_track.genre.clone();
        dest_track.album_artist = src_track.album_artist.clone();
        dest_track.composer = src_track.composer.clone();
        dest_track.track_number = src_track.track_number;
        dest_track.track_max = src_track.track_max;
        dest_track.disc_number = src_track.disc_number;
        dest_track.disc_max = src_track.disc_max;
        dest_track.release_date = src_track.release_date;
        dest_track.memo = src_track.memo.clone();
        dest_track.lyrics = src_track.lyrics.clone();
    }

    /// PCとDBの編集可能データの、全競合情報を出力
    fn display_all_conflicts(
        &self,
        conflicts: &[TrackItemConflict],
        pc_track: &TrackSync,
        db_track: &TrackSync,
    ) -> anyhow::Result<()> {
        let cui = &self.cui;

        for conflict in conflicts {
            let item_name = conflict.item_name();

            match conflict.item_kind {
                //メモと歌詞は省略
                TrackItemKind::Lyrics => {
                    cui_outln!(cui, "* {}が異なります", item_name)?;
                }
                _ => {
                    let pc_value = conflict.display_value(pc_track);
                    let db_value = conflict.display_value(db_track);
                    cui_outln!(
                        cui,
                        "* {}: {} | {}",
                        item_name,
                        pc_value.as_deref().unwrap_or("None"),
                        db_value.as_deref().unwrap_or("None"),
                    )?;
                }
            }
        }

        cui_outln!(cui, "(PC | DB)")?;
        cui_outln!(cui)?;

        Ok(())
    }

    /// アートワークの情報をコンソールに出力
    fn display_artwork(&self, artworks: &[TrackArtwork]) -> anyhow::Result<()> {
        let cui = &self.cui;

        if artworks.is_empty() {
            cui_outln!(cui, "アートワークなし")?;
            return Ok(());
        }
        for (idx, artwork) in artworks.iter().enumerate() {
            //画像タイプを文字列化
            let picture_type_str = match artwork.picture_type {
                0 => "Other",
                1 => "32x32 pixels 'file icon' (PNG only)",
                2 => "Other file icon",
                3 => "Cover (front)",
                4 => "Cover (back)",
                5 => "Leaflet page",
                6 => "Media (e.g. label side of CD)",
                7 => "Lead artist/lead performer/soloist",
                8 => "Artist/performer",
                9 => "Conductor",
                10 => "Band/Orchestra",
                11 => "Composer",
                12 => "Lyricist/text writer",
                13 => "Recording Location",
                14 => "During recording",
                15 => "During performance",
                16 => "Movie/video screen capture",
                17 => "A bright coloured fish",
                18 => "Illustration",
                19 => "Band/artist logotype",
                20 => "Publisher/Studio logotype",
                _ => "Unknown",
            };

            cui_outln!(cui, "- アートワーク {}", idx)?;
            cui_outln!(cui, "    MIME type: {}", artwork.picture.mime_type)?;
            cui_outln!(
                cui,
                "    Picture type: {} ({})",
                artwork.picture_type,
                picture_type_str
            )?;
            cui_outln!(cui, "    Description: {}", artwork.description)?;

            //TODO WalkBase1では画像を解析してwidth,heightを表示していた
            /*
            PictureDecoder decoder;
            unique_ptr<PictureFormat> picFormat = decoder.decode(songArtwork->artworkImage->image, songArtwork->artworkImage->mimeType);

            cout << "    Width: " << picFormat->getWidth() << endl;
            cout << "    Height: " << picFormat->getHeight() << endl;
            */
        }

        Ok(())
    }
}
