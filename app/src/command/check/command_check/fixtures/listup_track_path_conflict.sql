-- Test fixture for test_listup_track_path_conflict
-- This sets up the database state where:
-- - PC, DAP, and DB have different track files (conflict case)
-- - DB contains 3 tracks, including one that doesn't exist on PC/DAP (db1.flac)

-- Create folder structure
INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (1, 'test/', 'test', NULL),
    (2, 'test/hoge/', 'hoge', 1),
    (3, 'test/hoge/child/', 'child', 2);

-- Insert track records that match what the mock returns for conflict scenario
-- Only including essential columns for this test
INSERT INTO tracks (path, folder_id, title, duration) VALUES 
    ('test/hoge/child/track1.flac', 3, 'Track 1', 180),
    ('test/hoge/track2.flac', 2, 'Track 2', 180),
    ('test/hoge/db1.flac', 2, 'DB Only Track', 180);