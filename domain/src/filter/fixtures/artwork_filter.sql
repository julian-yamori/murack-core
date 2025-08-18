-- Test fixture for artwork filter tests
-- Sets up tracks with and without artwork

INSERT INTO tracks (id, duration, path, title) VALUES 
    (1, 180, 'track1.mp3', 'No Artwork'),
    (2, 180, 'track2.mp3', 'Single Artwork'),
    (3, 180, 'track3.mp3', 'Multiple Artworks');

-- Insert artwork relationships  
INSERT INTO track_artworks (track_id, order_index, artwork_id, picture_type, description) VALUES 
    (2, 0, 7, 3, ''),
    (3, 0, 5, 3, ''),
    (3, 1, 6, 4, '');