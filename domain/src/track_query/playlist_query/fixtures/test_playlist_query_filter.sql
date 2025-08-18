-- Filter プレイリストテスト用のデータ
-- rating >= 4 のフィルタ条件を設定

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
    (3, 220, '/music/track3.mp3', 'Track C', 'Track C', 'Artist C', 'Artist C', 'Album C', 'Album C', 'Jazz', 'Composer C', 'Composer C', 3, 1, '2023-03-01', 4, '2023-06-03 12:00:00');

-- Filter プレイリスト（rating >= 4）
INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, in_folder_order, filter_json) VALUES
    (2, 'filter', 'High Rated Songs', 'artist', false, false, 0, '{
        "target": "rating",
        "range": {
            "op": "large_equal",
            "value": 4
        }
    }');