-- Artist ソートテスト用のデータ

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
    (2, 200, '/music/track2.mp3', 'Track B', 'Track B', 'Artist C', 'Artist C', 'Album C', 'Album C', 'Pop', 'Composer C', 'Composer C', 2, 1, '2023-02-01', 3, '2023-06-02 11:00:00'),
    (3, 220, '/music/track3.mp3', 'Track C', 'Track C', 'Artist B', 'Artist B', 'Album B', 'Album B', 'Jazz', 'Composer B', 'Composer B', 3, 1, '2023-03-01', 4, '2023-06-03 12:00:00');

-- Artist ソートのプレイリスト
INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, in_folder_order) VALUES
    (1, 'normal', 'Test Playlist', 'artist', false, false, 0);

INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES
    (1, 0, 1),  -- Artist A
    (1, 1, 2),  -- Artist C  
    (1, 2, 3);  -- Artist B