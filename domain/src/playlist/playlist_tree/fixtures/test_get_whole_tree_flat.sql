-- Test fixture for flat playlist tree test
-- This sets up 3 playlists with no parent-child relationships

INSERT INTO playlists (id, playlist_type, name, parent_id, in_folder_order, sort_type, sort_desc, save_dap, listuped_flag, dap_changed) 
VALUES 
    (3, 'normal', 'one', NULL, 0, 'artist', false, true, false, true),
    (5, 'normal', 'two', NULL, 0, 'artist', false, true, false, true),
    (2, 'normal', 'three', NULL, 0, 'artist', false, true, false, true);