-- Test fixture for tags filter tests
-- Sets up songs with various tag combinations

INSERT INTO tracks (id, duration, path, title) VALUES 
    (1, 180, 'song1.mp3', 'No Tags Song'),
    (2, 180, 'song2.mp3', 'Tag 4 Only'),
    (3, 180, 'song3.mp3', 'Tags 4 and 83'),
    (4, 180, 'song4.mp3', 'Tags 8 and 83');

-- Insert tag relationships
INSERT INTO track_tags (track_id, tag_id) VALUES 
    (2, 4),
    (3, 4),
    (3, 83),
    (4, 8),
    (4, 83);