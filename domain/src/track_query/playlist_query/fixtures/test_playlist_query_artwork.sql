-- ArtworkId カラム取得テスト用のデータ
-- track_artworks テーブルとの JOIN をテストする

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

-- アートワークデータ
INSERT INTO artworks (id, hash, image, image_mini, mime_type) VALUES
    (1, 'hash1'::bytea, 'image1_data'::bytea, 'mini1_data'::bytea, 'image/jpeg'),
    (2, 'hash2'::bytea, 'image2_data'::bytea, 'mini2_data'::bytea, 'image/png');

-- track_artworks データ（track1 と track3 にアートワークを設定）
INSERT INTO track_artworks (track_id, artwork_id, order_index, picture_type, description) VALUES
    (1, 1, 0, 0, 'Cover Art'),  -- track1 の先頭アートワーク
    (1, 2, 1, 0, 'Back Cover'), -- track1 の2番目のアートワーク（テストでは使われない）
    (3, 2, 0, 0, 'Cover Art');  -- track3 の先頭アートワーク

INSERT INTO playlists (id, playlist_type, name, sort_type, sort_desc, listuped_flag, in_folder_order) VALUES
    (1, 'normal', 'Test Playlist', 'playlist', false, false, 0);

INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES
    (1, 0, 1),
    (1, 1, 2),
    (1, 2, 3);