-- Test fixture for string filter tests
-- Sets up tracks with various artist names for string filtering

INSERT INTO tracks (id, duration, path, title, artist) VALUES 
    (1, 180, 'track1.mp3', 'Title1', 'test'),
    (2, 180, 'track2.mp3', 'Title2', 'AAtest'),
    (3, 180, 'track3.mp3', 'Title3', 'testAA'),
    (4, 180, 'track4.mp3', 'Title4', 'AAtestAA'),
    (5, 180, 'track5.mp3', 'Title5', 'testAAtestAAtestAAtest'),
    (6, 180, 'track6.mp3', 'Title6', 'teAAst'),
    (7, 180, 'track7.mp3', 'Title7', 'AAAAAA'),
    (8, 180, 'track8.mp3', 'Title8', ''),
    (9, 180, 'track9.mp3', 'Title9', 'te%st'),
    (10, 180, 'track10.mp3', 'Title10', 'AAte%stAA');