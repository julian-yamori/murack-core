-- Test fixture for SortTypeWithPlaylist order_query tests
-- Creates tracks with various metadata for testing different sort orders

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
    (1, 180, '/music/album1/01.mp3', 'C Song', 'C Song', 'Artist B', 'Artist B', 'Album Z', 'Album Z', 'Rock', 'Composer A', 'Composer A', 1, 1, '2023-01-01', 5, '2023-06-01 10:00:00'),
    (2, 240, '/music/album2/02.mp3', 'A Song', 'A Song', 'Artist A', 'Artist A', 'Album A', 'Album A', 'Jazz', 'Composer B', 'Composer B', 2, 1, '2023-02-01', 3, '2023-06-02 11:00:00'),
    (3, 200, '/music/album3/03.mp3', 'B Song', 'B Song', 'Artist C', 'Artist C', 'Album B', 'Album B', 'Pop', 'Composer C', 'Composer C', 1, 2, '2023-03-01', 4, '2023-06-03 12:00:00'),
    (4, 160, '/music/album1/04.mp3', 'D Song', 'D Song', 'Artist A', 'Artist A', 'Album A', 'Album A', 'Rock', 'Composer A', 'Composer A', 3, 1, '2023-01-15', 2, '2023-06-04 13:00:00'),
    (5, 220, '/music/album2/05.mp3', 'E Song', 'E Song', 'Artist B', 'Artist B', 'Album B', 'Album B', 'Jazz', 'Composer B', 'Composer B', 2, 2, '2023-02-15', 5, '2023-06-05 14:00:00');

INSERT INTO playlists (id, playlist_type, name, in_folder_order, sort_type) VALUES
    (1, 'normal', 'Playlist A', 0, 'playlist');

INSERT INTO playlist_tracks (playlist_id, order_index, track_id) VALUES
    (1, 0, 1),
    (1, 1, 5),
    (1, 2, 2),
    (1, 3, 4);
