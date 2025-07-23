-- Test fixture for existing folder test  
-- This sets up the database with the target folder already existing

INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (2, 'test/hoge/', 'hoge', 1);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (12, 'test/hoge/fuga/', 'fuga', 2);