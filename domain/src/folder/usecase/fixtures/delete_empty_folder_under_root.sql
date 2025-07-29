-- Test fixture for test_delete_db_if_empty_by_id::delete_under_root()
--
-- This sets up the database state where:
-- - Folder 15 exists directly under root, empty (should be deleted)
-- - Parent is Root, so recursion stops after deleting folder 15

INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (15, 'music/', 'music', NULL);