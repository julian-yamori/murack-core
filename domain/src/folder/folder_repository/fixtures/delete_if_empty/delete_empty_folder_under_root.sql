-- Test fixture for delete_db_if_empty::delete_under_root()
--
-- This sets up the database state where:
-- - Folder 15 exists directly under root, empty (should be deleted)
-- - Parent is Root, so recursion stops after deleting folder 15

INSERT INTO folder_paths (id, path, name, parent_id) VALUES 
    (15, 'music/', 'music', NULL),
    (25, 'otherfolder/', 'otherfolder', NULL);
