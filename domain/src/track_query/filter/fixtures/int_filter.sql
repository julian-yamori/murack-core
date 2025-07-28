-- Test fixture for int filter tests (track_number)
-- Sets up tracks with various track numbers

INSERT INTO tracks (id, duration, path, title, track_number) VALUES 
    (1, 180, 'track1.mp3', 'No Track Number', NULL),
    (2, 180, 'track2.mp3', 'Track 1', 1),
    (3, 180, 'track3.mp3', 'Track 5', 5),
    (4, 180, 'track4.mp3', 'Track 9', 9),
    (5, 180, 'track5.mp3', 'Track 10', 10),
    (6, 180, 'track6.mp3', 'Track 25', 25),
    (7, 180, 'track7.mp3', 'Track 123', 123);