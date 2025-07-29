-- Test fixture for test_delete_db_if_empty_by_id::with_subfolders()
-- 
-- This sets up the database state where:
-- - Folder 15 exists but has subfolders (should not be deleted)

INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (15, 'music/', 'music', NULL),
    (20, 'music/subfolder/', 'subfolder', 15);