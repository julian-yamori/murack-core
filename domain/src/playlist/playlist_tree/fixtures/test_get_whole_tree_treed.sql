-- Test fixture for hierarchical playlist tree test
-- This sets up playlists with parent-child relationships:
-- root1 (5)
--   ├── 1-1 (3)
--   ├── 1-2 (2)
--   │   ├── 1-2-1 (9)
--   │   └── 1-2-2 (98)
--   └── 1-3 (1)
-- root2 (35)
--   └── 2-1 (75)

INSERT INTO playlists (id, playlist_type, name, parent_id, in_folder_order, sort_type, sort_desc, save_dap, listuped_flag, dap_changed) 
VALUES 
    (3, 'normal', '1-1', 5, 0, 'artist', false, true, false, true),
    (5, 'normal', 'root1', NULL, 0, 'artist', false, true, false, true),
    (9, 'normal', '1-2-1', 2, 0, 'artist', false, true, false, true),
    (2, 'normal', '1-2', 5, 0, 'artist', false, true, false, true),
    (35, 'normal', 'root2', NULL, 0, 'artist', false, true, false, true),
    (75, 'normal', '2-1', 35, 0, 'artist', false, true, false, true),
    (98, 'normal', '1-2-2', 2, 0, 'artist', false, true, false, true),
    (1, 'normal', '1-3', 5, 0, 'artist', false, true, false, true);