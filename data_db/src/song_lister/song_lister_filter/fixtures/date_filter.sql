-- Test fixture for date filter tests (release_date)
-- Sets up songs with various release dates

INSERT INTO tracks (id, duration, path, title, release_date) VALUES 
    (1, 180, 'song1.mp3', 'No Release Date', NULL),
    (2, 180, 'song2.mp3', 'Release 1998-12-10', '1998-12-10'),
    (3, 180, 'song3.mp3', 'Release 2012-04-05', '2012-04-05'),
    (4, 180, 'song4.mp3', 'Release 2021-09-26', '2021-09-26');