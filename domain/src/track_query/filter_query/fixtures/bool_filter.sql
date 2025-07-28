-- Test fixture for bool filter tests (suggest_target)
-- Sets up tracks with true/false suggest_target values

INSERT INTO tracks (id, duration, path, title, suggest_target) VALUES 
    (1, 180, 'track1.mp3', 'Suggest True', true),
    (2, 180, 'track2.mp3', 'Suggest False', false);