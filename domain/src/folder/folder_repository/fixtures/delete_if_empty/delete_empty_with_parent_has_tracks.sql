-- Test fixture for delete_db_if_empty::parent_has_tracks()
--
-- This sets up the database state where:
-- - Folder 4 exists and contains tracks (should not be deleted)
-- - Folder 15 exists as a child of folder 4, empty (should be deleted)

INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (4, 'music/', 'music', NULL),
    (15, 'music/empty/', 'empty', 4);

INSERT INTO tracks (id, path, folder_id, title, duration) VALUES 
    (1, 'music/song1.mp3', 4, 'Song 1', 180),
    (2, 'music/song2.mp3', 4, 'Song 2', 210);