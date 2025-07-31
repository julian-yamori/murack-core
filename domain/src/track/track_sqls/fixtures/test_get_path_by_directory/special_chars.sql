INSERT INTO folder_paths (id, path, name, parent_id) VALUES
    (1, 'test/', 'test', NULL),
    (2, 'test/d%i_r$/', 'd%i_r$', 1),
    (3, 'test/dZi_r$/', 'dZi_r$', 1),
    (4, 'dummy/', 'dummy', NULL),
    (5, 'dummy/test/', 'test', 4),
    (6, 'dummy/test/d%i_r$/', 'd%i_r$', 5);

INSERT INTO tracks (id, duration, path, folder_id, title) VALUES 
    (1, 180, 'test/d%i_r$/hoge.flac', 2, 'Track 1'),
    (2, 240, 'test/dZi_r$/dummy.flac', 3, 'Track 2'),
    (3, 200, 'fuga.flac', NULL, 'Track 3'),
    (4, 170, 'dummy/test/d%i_r$/dummy.mp3', 6, 'Track 4');
