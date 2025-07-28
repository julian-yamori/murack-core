-- Test fixture for tags filter tests
-- Sets up tracks with various tag combinations

INSERT INTO tracks (id, duration, path, title) VALUES 
    (1, 180, 'track1.mp3', 'No Tags Track'),
    (2, 180, 'track2.mp3', 'Tag 4 Only'),
    (3, 180, 'track3.mp3', 'Tags 4 and 83'),
    (4, 180, 'track4.mp3', 'Tags 8 and 83');

-- Insert tag relationships
INSERT INTO track_tags (track_id, tag_id) VALUES 
    (2, 4),
    (3, 4),
    (3, 83),
    (4, 8),
    (4, 83);