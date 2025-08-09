-- Rename sort_type enum to sort_type_with_playlist
-- This migration renames the enum type while preserving all existing data and references

-- First, create the new enum type with the desired name
CREATE TYPE sort_type_with_playlist AS ENUM (
    'track_name',
    'artist', 
    'album',
    'genre',
    'playlist',
    'composer',
    'duration',
    'track_index',
    'disc_index', 
    'release_date',
    'rating',
    'entry_date',
    'path'
);

-- Update the playlists table to use the new enum type
ALTER TABLE playlists 
    ALTER COLUMN sort_type TYPE sort_type_with_playlist 
    USING sort_type::text::sort_type_with_playlist;

-- Drop the old enum type
DROP TYPE sort_type;