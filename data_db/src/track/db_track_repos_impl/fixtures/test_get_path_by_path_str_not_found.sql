-- Test fixture for not found case test
-- This sets up an empty database (no tracks exist)

-- フォルダを作成
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (2, 'test/fuga/', 'fuga', 1);

-- test/hoge/ ディレクトリ内の検索にヒットしない楽曲を作成
INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'test/fuga/track1.mp3', 2, 'Track DUMMY');
