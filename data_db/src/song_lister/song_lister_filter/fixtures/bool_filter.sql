-- Test fixture for bool filter tests (suggest_target)
-- Sets up songs with true/false suggest_target values

INSERT INTO tracks (id, duration, path, title, suggest_target) VALUES 
    (1, 180, 'song1.mp3', 'Suggest True', true),
    (2, 180, 'song2.mp3', 'Suggest False', false);