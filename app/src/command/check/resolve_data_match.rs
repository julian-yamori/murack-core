use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;
use murack_core_domain::{
    Error as DomainError, FileLibraryRepository,
    artwork::{DbArtworkRepository, SongArtwork},
    check::CheckUsecase,
    db::DbTransaction,
    path::LibSongPath,
    song::SongItemKind,
    sync::{DbSongSync, DbSongSyncRepository, SongSync},
};
use sqlx::PgPool;

use super::{SongItemConflict, messages};
use crate::{Config, cui::Cui};

/// データ内容同一性についての解決処理
#[automock]
#[async_trait]
pub trait ResolveDataMatch {
    /// データ内容同一性についての解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve(&self, db_pool: &PgPool, song_path: &LibSongPath) -> Result<bool>;
}

/// ResolveDataMatchの実装
pub struct ResolveDataMatchImpl<CUI, FR, CS, AR, SSR>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository + Send + Sync,
    CS: CheckUsecase + Send + Sync,
    AR: DbArtworkRepository + Send + Sync,
    SSR: DbSongSyncRepository + Send + Sync,
{
    config: Arc<Config>,
    cui: Arc<CUI>,
    file_library_repository: FR,
    check_usecase: CS,
    db_artwork_repository: AR,
    db_song_sync_repository: SSR,
}

#[async_trait]
impl<CUI, FR, CS, AR, SSR> ResolveDataMatch for ResolveDataMatchImpl<CUI, FR, CS, AR, SSR>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository + Send + Sync,
    CS: CheckUsecase + Send + Sync,
    AR: DbArtworkRepository + Send + Sync,
    SSR: DbSongSyncRepository + Send + Sync,
{
    /// データ内容同一性についての解決処理
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve(&self, db_pool: &PgPool, song_path: &LibSongPath) -> Result<bool> {
        //データ読み込み
        let mut pc_data = self
            .file_library_repository
            .read_song_sync(&self.config.pc_lib, song_path)?;
        let mut db_data = self.load_db_song(db_pool, song_path).await?;

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

impl<CUI, FR, CS, AR, SSR> ResolveDataMatchImpl<CUI, FR, CS, AR, SSR>
where
    CUI: Cui + Send + Sync,
    FR: FileLibraryRepository + Send + Sync,
    CS: CheckUsecase + Send + Sync,
    AR: DbArtworkRepository + Send + Sync,
    SSR: DbSongSyncRepository + Send + Sync,
{
    pub fn new(
        config: Arc<Config>,
        cui: Arc<CUI>,
        file_library_repository: FR,
        check_usecase: CS,
        db_artwork_repository: AR,
        db_song_sync_repository: SSR,
    ) -> Self {
        Self {
            config,
            cui,
            file_library_repository,
            check_usecase,
            db_artwork_repository,
            db_song_sync_repository,
        }
    }

    /// PC・DB間の、曲情報(編集可能部)の齟齬の解決
    ///
    /// # Returns
    /// 次の解決処理へ継続するか
    async fn resolve_editable(
        &self,
        db_pool: &PgPool,
        pc_song: &mut SongSync,
        db_song: &mut DbSongSync,
    ) -> Result<bool> {
        //全体を比較して齟齬リストを取得
        let conflict_items = self
            .check_usecase
            .check_editable(pc_song, &db_song.song_sync);
        //齟齬がなければ次の処理へ
        if conflict_items.is_empty() {
            return Ok(true);
        }

        let conflicts: Vec<_> = conflict_items
            .into_iter()
            .map(|item_kind| SongItemConflict { item_kind })
            .collect();

        let cui = &self.cui;

        //結果表示
        cui_outln!(cui, "----")?;
        self.display_all_conflicts(&conflicts, pc_song, &db_song.song_sync)?;

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
                self.overwrite_song_editable(pc_song, &mut db_song.song_sync);
                self.save_db_exclude_artwork(db_pool, db_song).await?;

                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                let song_path = &db_song.path;

                //PCのデータを上書き
                self.overwrite_song_editable(&db_song.song_sync, pc_song);
                self.overwrite_pc_song_file(song_path, pc_song)?;

                //DAPのデータをPCのデータで上書き
                self.file_library_repository.overwrite_song_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    song_path,
                )?;

                Ok(true)
            }
            //それぞれの項目個別に処理方法を選択
            '3' => {
                for conflict in conflicts {
                    if !self
                        .resolve_each_property(db_pool, pc_song, db_song, &conflict)
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
        pc_song: &mut SongSync,
        db_song: &mut DbSongSync,
        conflict: &SongItemConflict,
    ) -> Result<bool> {
        let db_sync = &mut db_song.song_sync;

        let input = {
            let cui = &self.cui;

            cui_outln!(cui, "----")?;
            cui_outln!(cui, "{}", conflict.item_name())?;

            cui_outln!(cui, "[PC]")?;
            cui_outln!(
                cui,
                "{}",
                conflict.display_value(pc_song).as_deref().unwrap_or("None")
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
                conflict.copy_each_sync(pc_song, db_sync);
                self.save_db_exclude_artwork(db_pool, db_song).await?;

                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                //PCの値を上書きする
                conflict.copy_each_sync(db_sync, pc_song);

                let song_path = &db_song.path;
                self.overwrite_pc_song_file(song_path, pc_song)?;

                //DAPのデータをPCのデータで上書き
                self.file_library_repository.overwrite_song_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    song_path,
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
        pc_song: &mut SongSync,
        db_song: &mut DbSongSync,
    ) -> Result<bool> {
        //アートワークが一致したらスキップ
        if self
            .check_usecase
            .check_artwork(pc_song, &db_song.song_sync)
        {
            return Ok(true);
        }

        let cui = &self.cui;

        cui_outln!(cui, "----")?;
        //PCのアートワーク情報を表示
        cui_outln!(cui, "[PC]")?;
        self.display_artwork(&pc_song.artworks)?;
        cui_outln!(cui)?;

        //DBのアートワーク情報を表示
        cui_outln!(cui, "[DB]")?;
        self.display_artwork(&db_song.song_sync.artworks)?;
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
                let mut tx = DbTransaction::PgTransaction {
                    tx: db_pool.begin().await?,
                };

                //DBに上書き保存
                let song_id = db_song.id;

                self.db_artwork_repository
                    .register_song_artworks(&mut tx, song_id, &pc_song.artworks)
                    .await?;

                //念の為、保存した値で変数値を上書きしておく
                db_song.song_sync.artworks = self
                    .db_artwork_repository
                    .get_song_artworks(&mut tx, song_id)
                    .await?;

                tx.commit().await?;
                Ok(true)
            }
            //DBからPCへ上書きし、DAPも更新
            '2' => {
                //PCのデータを上書き
                pc_song.artworks = db_song.song_sync.artworks.clone();

                //PCに保存
                let song_path = &db_song.path;
                self.overwrite_pc_song_file(song_path, pc_song)?;

                //DAPのデータをPCのデータで上書き
                self.file_library_repository.overwrite_song_over_lib(
                    &self.config.pc_lib,
                    &self.config.dap_lib,
                    song_path,
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
        pc_song: &mut SongSync,
        db_song: &mut DbSongSync,
    ) -> Result<bool> {
        //再生時間が一致したらスキップ
        if self
            .check_usecase
            .check_duration(pc_song, &db_song.song_sync)
        {
            return Ok(true);
        }

        let input = {
            let cui = &self.cui;

            //再生時間の齟齬を表示
            cui_outln!(cui, "----")?;
            cui_outln!(
                cui,
                "* 再生時間: {}ms | {}ms",
                pc_song.duration,
                db_song.song_sync.duration
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
                db_song.song_sync.duration = pc_song.duration;
                self.save_db_exclude_artwork(db_pool, db_song).await?;

                Ok(true)
            }
            '0' => Ok(true),
            '-' => Ok(false),
            _ => unreachable!(),
        }
    }

    /// DBから曲ファイルを読み込み
    async fn load_db_song(&self, db_pool: &PgPool, song_path: &LibSongPath) -> Result<DbSongSync> {
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        self.db_song_sync_repository
            .get_by_path(&mut tx, song_path)
            .await?
            .ok_or_else(|| DomainError::DbSongNotFound(song_path.clone()).into())
    }

    /// DBに曲の連携情報(アートワーク以外)を保存
    async fn save_db_exclude_artwork(&self, db_pool: &PgPool, db_song: &DbSongSync) -> Result<()> {
        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        self.db_song_sync_repository
            .save_exclude_artwork(&mut tx, db_song)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// PCのファイルの曲データを上書き
    fn overwrite_pc_song_file(&self, song_path: &LibSongPath, pc_song: &SongSync) -> Result<()> {
        self.file_library_repository.overwrite_song_sync(
            &self.config.pc_lib,
            song_path,
            pc_song,
        )?;

        Ok(())
    }

    /// 曲の編集可能データを上書き
    ///
    /// # todo
    /// domainに移動
    fn overwrite_song_editable(&self, src_song: &SongSync, dest_song: &mut SongSync) {
        dest_song.title = src_song.title.clone();
        dest_song.artist = src_song.artist.clone();
        dest_song.album = src_song.album.clone();
        dest_song.genre = src_song.genre.clone();
        dest_song.album_artist = src_song.album_artist.clone();
        dest_song.composer = src_song.composer.clone();
        dest_song.track_number = src_song.track_number;
        dest_song.track_max = src_song.track_max;
        dest_song.disc_number = src_song.disc_number;
        dest_song.disc_max = src_song.disc_max;
        dest_song.release_date = src_song.release_date;
        dest_song.memo = src_song.memo.clone();
        dest_song.lyrics = src_song.lyrics.clone();
    }

    /// PCとDBの編集可能データの、全競合情報を出力
    fn display_all_conflicts(
        &self,
        conflicts: &[SongItemConflict],
        pc_song: &SongSync,
        db_song: &SongSync,
    ) -> anyhow::Result<()> {
        let cui = &self.cui;

        for conflict in conflicts {
            let item_name = conflict.item_name();

            match conflict.item_kind {
                //メモと歌詞は省略
                SongItemKind::Lyrics => {
                    cui_outln!(cui, "* {}が異なります", item_name)?;
                }
                _ => {
                    let pc_value = conflict.display_value(pc_song);
                    let db_value = conflict.display_value(db_song);
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
    fn display_artwork(&self, artworks: &[SongArtwork]) -> anyhow::Result<()> {
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
