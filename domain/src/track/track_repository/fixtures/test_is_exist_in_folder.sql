INSERT INTO folder_paths (id, path, name, parent_id) VALUES
    (11, 'test/', 'test', NULL),
    (22, 'folder2/', 'folder2', NULL),
    (99, 'empty/', 'empty', NULL);

INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 123, 'test/hoge.flac', 11, '曲名1'),
    (2, 123, 'folder2/fuga.flac', 22, '曲名2'),
    (3, 123, 'test/piyo.flac', 11, '曲名3'),
    (4, 123, 'piyo.flac', NULL, '曲名4');