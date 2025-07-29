-- Test fixture for test_register_db::to_root_folder
-- This sets up the database state where:
-- - Empty database (track will be added directly to root)
-- - Some playlists exist with listuped_flag = true to test reset functionality

INSERT INTO playlists (id, playlist_type, name, parent_id, in_folder_order, filter_json, sort_type, sort_desc, save_dap, listuped_flag, dap_changed) VALUES 
    (1, 'filter', 'Test Filter Playlist', NULL, 1, '{}', 'artist', false, true, true, false),
    (2, 'folder', 'Test Folder Playlist', NULL, 2, NULL, 'artist', false, true, true, false);