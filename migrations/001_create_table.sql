-- Create enum types for PostgreSQL
CREATE TYPE playlist_type AS ENUM ('normal', 'filter', 'folder');

CREATE TYPE sort_type AS ENUM (
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

-- Tag groups table
CREATE TABLE tag_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    order_index INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Tags table
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    group_id INTEGER NOT NULL REFERENCES tag_groups(id),
    order_index INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Folder paths table
CREATE TABLE folder_paths (
    id SERIAL PRIMARY KEY,
    path VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    parent_id INTEGER REFERENCES folder_paths(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Tracks table
CREATE TABLE tracks (
    id SERIAL PRIMARY KEY,
    duration INTEGER NOT NULL,
    path VARCHAR NOT NULL,
    folder_id INTEGER REFERENCES folder_paths(id),
    title VARCHAR NOT NULL,

    -- このあたりの並び順は、この機に GUI に合わせて整理する
    artist VARCHAR NOT NULL DEFAULT '',
    album_artist VARCHAR NOT NULL DEFAULT '',
    album VARCHAR NOT NULL DEFAULT '',
    composer VARCHAR NOT NULL DEFAULT '',
    genre VARCHAR NOT NULL DEFAULT '',

    track_number INTEGER,
    track_max INTEGER,
    disc_number INTEGER,
    disc_max INTEGER,
    release_date DATE,
    rating SMALLINT NOT NULL DEFAULT 0,
    original_track VARCHAR NOT NULL DEFAULT '',
    suggest_target BOOLEAN NOT NULL DEFAULT true,
    memo TEXT NOT NULL DEFAULT '',
    memo_manage TEXT NOT NULL DEFAULT '',
    lyrics TEXT NOT NULL DEFAULT '',
    title_order VARCHAR NOT NULL DEFAULT '',
    artist_order VARCHAR NOT NULL DEFAULT '',
    album_artist_order VARCHAR NOT NULL DEFAULT '',
    album_order VARCHAR NOT NULL DEFAULT '',
    composer_order VARCHAR NOT NULL DEFAULT '',
    genre_order VARCHAR NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Playlists table (with embedded filter JSON)
CREATE TABLE playlists (
    id SERIAL PRIMARY KEY,
    playlist_type playlist_type NOT NULL,
    name VARCHAR NOT NULL,
    parent_id INTEGER REFERENCES playlists(id),
    in_folder_order INTEGER NOT NULL,
    filter_json JSONB,
    sort_type sort_type NOT NULL,
    sort_desc BOOLEAN NOT NULL DEFAULT false,
    save_dap BOOLEAN NOT NULL DEFAULT false,
    listuped_flag BOOLEAN NOT NULL DEFAULT false,
    dap_changed BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Search presets table (with embedded filter JSON)
CREATE TABLE search_presets (
    id SERIAL PRIMARY KEY,
    order_index INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    filter_json JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Artwork table
CREATE TABLE artworks (
    id SERIAL PRIMARY KEY,
    hash BYTEA NOT NULL,
    image BYTEA NOT NULL,
    image_mini BYTEA NOT NULL,
    mime_type VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Track artwork association table
CREATE TABLE track_artworks (
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    order_index INTEGER NOT NULL,
    artwork_id INTEGER NOT NULL REFERENCES artworks(id),
    picture_type INTEGER NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (track_id, order_index)
);

-- Track tags association table
CREATE TABLE track_tags (
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (track_id, tag_id)
);

-- Playlist tracks association table
CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id),
    order_index INTEGER NOT NULL,
    track_id INTEGER NOT NULL REFERENCES tracks(id),
    PRIMARY KEY (playlist_id, order_index)
);

-- Create function for updating updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for automatic updated_at updates
CREATE TRIGGER update_tag_groups_updated_at BEFORE UPDATE ON tag_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_folder_paths_updated_at BEFORE UPDATE ON folder_paths
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tracks_updated_at BEFORE UPDATE ON tracks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_playlists_updated_at BEFORE UPDATE ON playlists
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_search_presets_updated_at BEFORE UPDATE ON search_presets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_artworks_updated_at BEFORE UPDATE ON artworks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_track_artworks_updated_at BEFORE UPDATE ON track_artworks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create indexes for performance
CREATE INDEX idx_tracks_folder_id ON tracks(folder_id);
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_album ON tracks(album);
CREATE INDEX idx_tracks_genre ON tracks(genre);
CREATE INDEX idx_playlists_parent_id ON playlists(parent_id);
CREATE INDEX idx_tags_group_id ON tags(group_id);
CREATE INDEX idx_folder_paths_parent_id ON folder_paths(parent_id);
CREATE INDEX idx_artworks_hash ON artworks(hash);
CREATE INDEX idx_track_tags_tag_id ON track_tags(tag_id);