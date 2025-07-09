//! DBのテーブルを用意

use crate::sql_func;
use anyhow::Result;
use domain::db_wrapper::TransactionWrapper;

pub fn artwork(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE `artwork` ( `rowid` integer, `hash` blob NOT NULL, `image` blob NOT NULL, `image_mini` blob, `mime_type` text NOT NULL, PRIMARY KEY(`rowid`) )",
        [],
    )
}

pub fn filter(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [filter] ( [rowid] integer, [parent_id] integer, [in_parent_order] integer NOT NULL, [root_id] integer NOT NULL, [target] integer NOT NULL, [str_value] text NOT NULL, [str_value2] text NOT NULL, [range] integer NOT NULL,  PRIMARY KEY([rowid]) )",
        [],
    )
}

pub fn folder_path(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [folder_path]([id] integer unique,[path] text primary key,[name] text not null,[parent_id] integer)",
        [],
    )
}

pub fn playlist(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [playlist]([rowid] integer primary key,[type] integer not null,[name] text not null,[parent_id] integer,[in_folder_order] integer not null,[filter_root] integer,[sort_type] integer not null,[sort_desc] integer not null,[save_dap] integer not null,[listuped_flag] integer not null,[dap_changed] integer not null)",
        [],
    )
}

pub fn playlist_song(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [playlist_song]([playlist_id] integer not null,[order] integer not null,[song_id] integer not null,primary key([playlist_id],[order]))",
        [],
    )
}

pub fn search_preset(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [search_preset] ( [rowid] integer, [order] integer NOT NULL, [name] text NOT NULL, [filter_root] integer NOT NULL,  PRIMARY KEY([rowid]) )",
        [],
    )
}

pub fn song(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [song] ( [rowid] integer, [duration] integer NOT NULL, [path] text NOT NULL, [folder_id] integer, [title] text NOT NULL, [artist] text NOT NULL, [album] text NOT NULL, [genre] text NOT NULL, [album_artist] text NOT NULL, [composer] text NOT NULL, [track_number] integer, [track_max] integer, [disc_number] integer, [disc_max] integer, [release_date] text, [memo] text NOT NULL, [rating] integer NOT NULL, [suggest_target] integer NOT NULL, [memo_manage] text NOT NULL, [lyrics] text NOT NULL, [title_order] text NOT NULL, [artist_order] text NOT NULL, [album_order] text NOT NULL, [album_artist_order] text NOT NULL, [composer_order] text NOT NULL, [genre_order] text NOT NULL, [entry_date] text NOT NULL, [original_song] text NOT NULL,  PRIMARY KEY([rowid]) )",
        [],
    )
}

pub fn song_artwork(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE `song_artwork` ( `song_id` integer NOT NULL, `order` integer NOT NULL, `artwork_id` integer NOT NULL, `picture_type` INTEGER NOT NULL, `description` integer NOT NULL, PRIMARY KEY(`song_id`,`order`) )",
        [],
    )
}

pub fn song_tags(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [song_tags]([song_id] integer not null,[tag_id] integer not null,primary key([song_id],[tag_id]))",
        [],
    )
}

pub fn tag(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [tag] ( [rowid] integer, [name] text NOT NULL, [group] integer NOT NULL, [order] integer NOT NULL, [description] text NOT NULL,  PRIMARY KEY([rowid]) )",
        [],
    )
}

pub fn tag_group(tx: &TransactionWrapper) -> Result<()> {
    sql_func::execute(
        tx,
        "CREATE TABLE [tag_group] ( [rowid] integer, [name] text NOT NULL, [order] integer NOT NULL, [description] text NOT NULL,  PRIMARY KEY([rowid]) )",
        [],
    )
}
