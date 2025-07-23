-- Test fixture for root directory search test  
-- This sets up songs in various directories for root search

-- フォルダを作成
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (2, 'test/hoge/', 'hoge', 1);

-- 様々な場所に楽曲を作成（ルート検索でヒットするもの）
INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'song1.mp3', NULL, 'Song 1'),
    (2, 240, 'test/hoge/song2.flac', 2, 'Song 2'),
    (3, 200, 'test/song3.m4a', 1, 'Song 3');