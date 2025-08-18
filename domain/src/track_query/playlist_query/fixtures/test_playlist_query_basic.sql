-- 基本的な PlaylistQuery::fetch() テスト用のデータ
-- Normal プレイリストとその曲データ

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
    (1, 180, '/music/track1.mp3', 'Track A', 'Track A', 'Artist A', 'Artist A', 'Album A', 'Album A', 'Rock', 'Composer A', 'Composer A', 1, 1, '2023-01-01', 5, '2023-06-01 10:00:00'),
    (2, 200, '/music/track2.mp3', 'Track B', 'Track B', 'Artist B', 'Artist B', 'Album B', 'Album B', 'Pop', 'Composer B', 'Composer B', 2, 1, '2023-02-01', 3, '2023-06-02 11:00:00'),
    (3, 220, '/music/track3.mp3', 'Track C', 'Track C', 'Artist C', 'Artist C', 'Album C', 'Album C', 'Jazz', 'Composer C', 'Composer C', 3, 1, '2023-03-01', 4, '2023-06-03 12:00:00'),
    (4, 240, '/music/track4.mp3', 'Track D', 'Track D', 'Artist D', 'Artist D', 'Album D', 'Album D', 'Jazz', 'Composer D', 'Composer D', 4, 1, '2023-04-01', 4, '2023-06-04 13:00:00');

INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, in_folder_order) VALUES
    (1, 'normal', 'Test Playlist', 'playlist', false, false, 0),
    (2, 'normal', 'Test Playlist 2', 'playlist', false, false, 1);

INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES
    -- Playlist 1
    (1, 0, 1),
    (1, 1, 2),
    (1, 2, 3),

    -- Playlist 2
    (2, 0, 2),
    (2, 1, 4);