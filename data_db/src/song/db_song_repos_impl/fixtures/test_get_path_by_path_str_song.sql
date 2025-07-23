-- Test fixture for specific song file search test
-- This sets up a specific song file "test/hoge.flac"

-- フォルダを作成
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);

INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    -- 検索するパスに一致する楽曲ファイルを作成
    (1, 300, 'test/hoge.flac', 1, 'Hoge Song'),
    -- 同じフォルダに、検索にヒットしない楽曲ファイルを作成
    (2, 300, 'test/fuga.flac', 1, 'Fuga Song');