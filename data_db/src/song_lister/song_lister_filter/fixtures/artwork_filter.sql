-- Test fixture for artwork filter tests
-- Sets up songs with and without artwork

INSERT INTO tracks (id, duration, path, title) VALUES 
    (1, 180, 'song1.mp3', 'No Artwork'),
    (2, 180, 'song2.mp3', 'Single Artwork'),
    (3, 180, 'song3.mp3', 'Multiple Artworks');

-- Insert artwork relationships  
INSERT INTO track_artworks (track_id, order_index, artwork_id, picture_type, description) VALUES 
    (2, 0, 7, 3, ''),
    (3, 0, 5, 3, ''),
    (3, 1, 6, 4, '');