INSERT INTO folder_paths (id, path, name, parent_id) VALUES
    (1, 'test/', 'test', NULL),
    (2, 'test/dir/', 'dir', 1),
    (3, 'dummy/', 'dummy', NULL),
    (4, 'dummy/test/', 'test', 3),
    (5, 'dummy/test/dir/', 'dir', 4);

INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'test/hoge.flac', 1, 'Song 1'),
    (2, 240, 'test/hoge2.flac', 1, 'Song 2'),
    (3, 200, 'fuga.flac', NULL, 'Song 3'),
    (4, 170, 'dummy/fuga.flac', 3, 'Song 4'),
    (5, 230, 'test/dir/hoge3.flac', 2, 'Song 5'),
    (6, 190, 'dummy/test/dir/dummy.mp3', 5, 'Song 6');
