-- Test fixture for test_listup_track_path_green
-- This sets up the database state where:
-- - PC, DAP, and DB all have the same track files (green/normal case)
-- - DB contains 4 tracks under test/hoge/ path

-- Create folder structure
INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (1, 'test/', 'test', NULL),
    (2, 'test/hoge/', 'hoge', 1),
    (3, 'test/hoge/child/', 'child', 2);

-- Insert track records that match what the mock returns
-- Only including essential columns for this test
INSERT INTO tracks (path, folder_id, title, duration) VALUES 
    ('test/hoge/track1.flac', 2, 'Track 1', 180),
    ('test/hoge/track2.flac', 2, 'Track 2', 180),
    ('test/hoge/child/track3.flac', 3, 'Track 3', 180),
    ('test/hoge/child/track4.flac', 3, 'Track 4', 180);