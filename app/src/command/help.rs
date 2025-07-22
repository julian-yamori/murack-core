use anyhow::Result;

use crate::cui::Cui;

/// ヘルプ出力コマンド
pub struct CommandHelp<CUI>
where
    CUI: Cui,
{
    cui: CUI,
}

impl<CUI> CommandHelp<CUI>
where
    CUI: Cui,
{
    pub fn new(_command_line: &[String], cui: CUI) -> Result<Self> {
        Ok(Self { cui })
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        let cui = &self.cui;

        cui_outln!(cui, "[サブコマンド一覧]\n")?;

        cui_outln!(cui, "playlist")?;
        cui_outln!(cui, "    DAPのプレイリストを更新")?;
        cui_outln!(cui)?;

        cui_outln!(cui, "add <ライブラリ内パス>")?;
        cui_outln!(cui, "    曲をライブラリに追加。")?;
        cui_outln!(cui, "    (DBにデータを追加し、PCからDAPにファイルをコピー)")?;
        cui_outln!(cui)?;

        cui_outln!(cui, "check |<options>| |<ライブラリ内パス>|")?;
        cui_outln!(
            cui,
            "    PC・DB・DAP間で問題がないか確認し、解決処理を行う。"
        )?;
        cui_outln!(cui, "    [options]")?;
        cui_outln!(
            cui,
            "    -i : PC・DAP間でファイル内容の比較を行わない(存在確認のみ行う)"
        )?;
        cui_outln!(cui)?;

        cui_outln!(cui, "move <移動元パス> <移動先パス>")?;
        cui_outln!(cui, "    ライブラリ内で曲のパスを移動する。")?;
        cui_outln!(cui)?;

        cui_outln!(cui, "remove <ライブラリ内パス>")?;
        cui_outln!(cui, "    曲データをライブラリから削除する。")?;
        cui_outln!(cui)?;

        cui_outln!(cui, "aw-get <音声絶対パス> |<画像保存先パス>|")?;
        cui_outln!(cui, "    オーディオファイルから画像を取得する。")?;
        cui_outln!(cui)?;

        cui_outln!(cui, "replace <音声絶対パス> <ライブラリ内パス>")?;
        cui_outln!(cui, "    ※未実装")?;
        cui_outln!(cui, "    オーディオファイルを置き換える。")?;
        cui_outln!(cui, "    ただしメタデータはライブラリの情報を維持する。")?;
        cui_outln!(cui)?;

        Ok(())
    }
}
