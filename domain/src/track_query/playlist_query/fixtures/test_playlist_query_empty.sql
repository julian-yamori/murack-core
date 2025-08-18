-- 空のプレイリストテスト用のデータ

INSERT INTO tracks (
    id, 
    duration, 
    path, 
    title, 
    title_order,
    artist, 
    artist_order,
    album,
    album_order,
    genre,
    composer,
    composer_order,
    track_number,
    disc_number,
    release_date,
    rating,
    created_at
) VALUES 
    (1, 180, '/music/track1.mp3', 'Track A', 'Track A', 'Artist A', 'Artist A', 'Album A', 'Album A', 'Rock', 'Composer A', 'Composer A', 1, 1, '2023-01-01', 5, '2023-06-01 10:00:00');

-- 空のプレイリスト（playlist_tracks にエントリなし）
INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, in_folder_order) VALUES
    (1, 'normal', 'Empty Playlist', 'playlist', false, false, 0);