-- Test fixture for string filter tests
-- Sets up songs with various artist names for string filtering

INSERT INTO tracks (id, duration, path, title, artist) VALUES 
    (1, 180, 'song1.mp3', 'Title1', 'test'),
    (2, 180, 'song2.mp3', 'Title2', 'AAtest'),
    (3, 180, 'song3.mp3', 'Title3', 'testAA'),
    (4, 180, 'song4.mp3', 'Title4', 'AAtestAA'),
    (5, 180, 'song5.mp3', 'Title5', 'testAAtestAAtestAAtest'),
    (6, 180, 'song6.mp3', 'Title6', 'teAAst'),
    (7, 180, 'song7.mp3', 'Title7', 'AAAAAA'),
    (8, 180, 'song8.mp3', 'Title8', ''),
    (9, 180, 'song9.mp3', 'Title9', 'te%st'),
    (10, 180, 'song10.mp3', 'Title10', 'AAte%stAA');