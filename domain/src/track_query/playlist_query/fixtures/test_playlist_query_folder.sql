-- Folder プレイリストテスト用のデータ
-- 親プレイリストと子プレイリストの構造

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
    (4, 240, '/music/track4.mp3', 'Track D', 'Track D', 'Artist D', 'Artist D', 'Album D', 'Album D', 'Blues', 'Composer D', 'Composer D', 4, 1, '2023-04-01', 2, '2023-06-04 13:00:00'),
    (5, 260, '/music/track5.mp3', 'Track E', 'Track E', 'Artist E', 'Artist E', 'Album E', 'Album E', 'Blues', 'Composer E', 'Composer E', 5, 1, '2023-05-01', 1, '2023-06-05 14:00:00');

INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, parent_id, in_folder_order) VALUES
    -- 親 Folder プレイリスト
    (3, 'folder', 'Music Folder', 'artist', false, false, NULL, 0),
    -- 子プレイリスト 1
    (4, 'normal', 'Child Playlist 1', 'playlist', false, true, 3, 1),
    -- 子プレイリスト 2
    (5, 'normal', 'Child Playlist 2', 'playlist', false, true, 3, 2),
    -- 無関係なプレイリスト
    (6, 'normal', 'Dummy Playlist', 'playlist', false, true, NULL, 1);

INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES
    -- 子プレイリスト 1 の曲
    (4, 0, 1),
    (4, 1, 2),

    -- 子プレイリスト 2 の曲（track2 は重複、track3, track4 は追加）
    (5, 0, 2),
    (5, 1, 3),
    (5, 2, 4),

    -- 無関係なプレイリストの曲
    (6, 0, 5),
    (6, 1, 1);
