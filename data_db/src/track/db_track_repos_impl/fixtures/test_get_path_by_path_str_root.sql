-- Test fixture for root directory search test  
-- This sets up tracks in various directories for root search

-- フォルダを作成
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (1, 'test/', 'test', NULL);
INSERT INTO folder_paths (id, path, name, parent_id) VALUES (2, 'test/hoge/', 'hoge', 1);

-- 様々な場所に楽曲を作成（ルート検索でヒットするもの）
INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'track1.mp3', NULL, 'Track 1'),
    (2, 240, 'test/hoge/track2.flac', 2, 'Track 2'),
    (3, 200, 'test/track3.m4a', 1, 'Track 3');