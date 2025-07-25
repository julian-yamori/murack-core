-- Test fixture for group filter tests (complex AND/OR combinations)
-- Sets up tracks for testing complex filter groups

INSERT INTO tracks (id, duration, path, title, artist, rating, release_date) VALUES 
    (1, 180, 'track1.mp3', 'Taro Track 1', 'taro', 3, NULL),
    (2, 180, 'track2.mp3', 'Jiro Track', 'jiro', 4, '2021-09-25'),
    (3, 180, 'track3.mp3', 'Taro Track 2', 'taro', 5, '1999-09-09'),
    (4, 180, 'track4.mp3', 'Taro Track 3', 'taro', 0, NULL),
    (5, 180, 'track5.mp3', '3bro Track', '3bro', 2, '1999-09-09'),
    (6, 180, 'track6.mp3', 'Taro Track 4', 'taro', 0, '2021-09-25'),
    (7, 180, 'track7.mp3', 'Taro Track 5', 'taro', 4, '2021-09-25');

-- Insert tag relationships for complex filtering
INSERT INTO track_tags (track_id, tag_id) VALUES 
    (1, 45),
    (1, 58),
    (2, 45),
    (2, 58),
    (4, 8),
    (4, 9),
    (4, 10),
    (6, 999),
    (7, 8),
    (7, 9),
    (7, 10);