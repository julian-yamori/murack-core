-- Test fixture for directory path search test
-- This sets up songs in the "test/hoge/" directory

-- フォルダを作成
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (2, 'test/hoge/', 'hoge', 1);

-- test/hoge/ ディレクトリ内の楽曲を作成
INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'test/hoge/song1.mp3', 2, 'Song 1'),
    (2, 240, 'test/hoge/song2.flac', 2, 'Song 2'),
    (3, 200, 'test/hoge/song3.m4a', 2, 'Song 3');

-- test/hoge/ の検索にヒットしない楽曲を作成
INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (4, 180, 'test/fuga.mp3', 1, 'Song DUMMY');
