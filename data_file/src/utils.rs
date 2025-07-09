use std::path::{Path, PathBuf};

/// パスの拡張子が音声ファイルのものか確認
pub fn is_audio_ext(path: &Path) -> bool {
    const AUDIO_FILE_EXTS: [&str; 7] = ["flac", "mp3", "m4a", "aac", "ogg", "wma", "wav"];

    let ext_os = match path.extension() {
        Some(e) => e,
        None => return false,
    };
    let ext = match ext_os.to_str() {
        Some(s) => s,
        None => return false,
    };

    AUDIO_FILE_EXTS.contains(&ext)
}

/// オーディオファイルに対応する.lrcファイルのパスを取得
pub fn get_lrc_path(audio: &Path) -> PathBuf {
    audio.with_extension("lrc")
}
