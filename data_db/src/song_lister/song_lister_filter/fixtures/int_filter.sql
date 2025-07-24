-- Test fixture for int filter tests (track_number)
-- Sets up songs with various track numbers

INSERT INTO tracks (id, duration, path, title, track_number) VALUES 
    (1, 180, 'song1.mp3', 'No Track Number', NULL),
    (2, 180, 'song2.mp3', 'Track 1', 1),
    (3, 180, 'song3.mp3', 'Track 5', 5),
    (4, 180, 'song4.mp3', 'Track 9', 9),
    (5, 180, 'song5.mp3', 'Track 10', 10),
    (6, 180, 'song6.mp3', 'Track 25', 25),
    (7, 180, 'song7.mp3', 'Track 123', 123);