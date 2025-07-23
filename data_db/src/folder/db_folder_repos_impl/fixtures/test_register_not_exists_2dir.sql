-- Test fixture for test_register_not_exists_2dir
-- This sets up the database state where:
-- - "test" folder exists
-- - "test/hoge" and "test/hoge/fuga" do not exist

INSERT INTO folder_paths (path, name, parent_id) VALUES ('test/', 'test', NULL);