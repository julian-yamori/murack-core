use super::DakuCnv;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// 並び替え用文字への変換マップ
pub static CHAR_ORDER_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut m = HashMap::new();

    //アルファベット大文字→小文字
    m.insert('A', 'a');
    m.insert('B', 'b');
    m.insert('C', 'c');
    m.insert('D', 'd');
    m.insert('E', 'e');
    m.insert('F', 'f');
    m.insert('G', 'g');

    m.insert('H', 'h');
    m.insert('I', 'i');
    m.insert('J', 'j');
    m.insert('K', 'k');
    m.insert('L', 'l');
    m.insert('M', 'm');
    m.insert('N', 'n');

    m.insert('O', 'o');
    m.insert('P', 'p');
    m.insert('Q', 'q');
    m.insert('R', 'r');
    m.insert('S', 's');
    m.insert('T', 't');
    m.insert('U', 'u');

    m.insert('V', 'v');
    m.insert('W', 'w');
    m.insert('X', 'x');
    m.insert('Y', 'y');
    m.insert('Z', 'z');

    //カタカナ→ひらがな
    m.insert('ァ', 'ぁ');
    m.insert('ィ', 'ぃ');
    m.insert('ゥ', 'ぅ');
    m.insert('ェ', 'ぇ');
    m.insert('ォ', 'ぉ');

    m.insert('ア', 'あ');
    m.insert('イ', 'い');
    m.insert('ウ', 'う');
    m.insert('エ', 'え');
    m.insert('オ', 'お');

    m.insert('カ', 'か');
    m.insert('キ', 'き');
    m.insert('ク', 'く');
    m.insert('ケ', 'け');
    m.insert('コ', 'こ');

    m.insert('ガ', 'が');
    m.insert('ギ', 'ぎ');
    m.insert('グ', 'ぐ');
    m.insert('ゲ', 'げ');
    m.insert('ゴ', 'ご');

    m.insert('サ', 'さ');
    m.insert('シ', 'し');
    m.insert('ス', 'す');
    m.insert('セ', 'せ');
    m.insert('ソ', 'そ');

    m.insert('ザ', 'ざ');
    m.insert('ジ', 'じ');
    m.insert('ズ', 'ず');
    m.insert('ゼ', 'ぜ');
    m.insert('ゾ', 'ぞ');

    m.insert('タ', 'た');
    m.insert('チ', 'ち');
    m.insert('ツ', 'つ');
    m.insert('テ', 'て');
    m.insert('ト', 'と');
    m.insert('ッ', 'っ');

    m.insert('ダ', 'だ');
    m.insert('ヂ', 'ぢ');
    m.insert('ヅ', 'づ');
    m.insert('デ', 'で');
    m.insert('ド', 'ど');

    m.insert('ナ', 'な');
    m.insert('ニ', 'に');
    m.insert('ヌ', 'ぬ');
    m.insert('ネ', 'ね');
    m.insert('ノ', 'の');

    m.insert('ハ', 'は');
    m.insert('ヒ', 'ひ');
    m.insert('フ', 'ふ');
    m.insert('ヘ', 'へ');
    m.insert('ホ', 'ほ');

    m.insert('バ', 'ば');
    m.insert('ビ', 'び');
    m.insert('ブ', 'ぶ');
    m.insert('ベ', 'べ');
    m.insert('ボ', 'ぼ');

    m.insert('パ', 'ぱ');
    m.insert('ピ', 'ぴ');
    m.insert('プ', 'ぷ');
    m.insert('ペ', 'ぺ');
    m.insert('ポ', 'ぽ');

    m.insert('マ', 'ま');
    m.insert('ミ', 'み');
    m.insert('ム', 'む');
    m.insert('メ', 'め');
    m.insert('モ', 'も');

    m.insert('ャ', 'ゃ');
    m.insert('ュ', 'ゅ');
    m.insert('ョ', 'ょ');

    m.insert('ヤ', 'や');
    m.insert('ユ', 'ゆ');
    m.insert('ヨ', 'よ');

    m.insert('ラ', 'ら');
    m.insert('リ', 'り');
    m.insert('ル', 'る');
    m.insert('レ', 'れ');
    m.insert('ロ', 'ろ');

    m.insert('ヮ', 'ゎ');

    m.insert('ワ', 'わ');
    m.insert('ヰ', 'ゐ');
    m.insert('ヱ', 'ゑ');
    m.insert('ヲ', 'を');

    m.insert('ン', 'ん');

    m.insert('ヽ', 'ゝ');
    m.insert('ヾ', 'ゞ');

    //半角カタカナ→ひらがな
    //濁音の可能性のある文字はms_HanKanaMapで定義
    m.insert('ｧ', 'ぁ');
    m.insert('ｨ', 'ぃ');
    m.insert('ｩ', 'ぅ');
    m.insert('ｪ', 'ぇ');
    m.insert('ｫ', 'ぉ');

    m.insert('ｱ', 'あ');
    m.insert('ｲ', 'い');
    m.insert('ｳ', 'う');
    m.insert('ｴ', 'え');
    m.insert('ｵ', 'お');

    m.insert('ｯ', 'っ');

    m.insert('ﾅ', 'な');
    m.insert('ﾆ', 'に');
    m.insert('ﾇ', 'ぬ');
    m.insert('ﾈ', 'ね');
    m.insert('ﾉ', 'の');

    m.insert('ﾏ', 'ま');
    m.insert('ﾐ', 'み');
    m.insert('ﾑ', 'む');
    m.insert('ﾒ', 'め');
    m.insert('ﾓ', 'も');

    m.insert('ｬ', 'ゃ');
    m.insert('ｭ', 'ゅ');
    m.insert('ｮ', 'ょ');

    m.insert('ﾔ', 'や');
    m.insert('ﾕ', 'ゆ');
    m.insert('ﾖ', 'よ');

    m.insert('ﾗ', 'ら');
    m.insert('ﾘ', 'り');
    m.insert('ﾙ', 'る');
    m.insert('ﾚ', 'れ');
    m.insert('ﾛ', 'ろ');

    m.insert('ﾜ', 'わ');
    m.insert('ｦ', 'を');
    m.insert('ﾝ', 'ん');

    m
});

/// 並び替え用文字への変換マップ(半角カタカナの濁音判別用
pub static HAN_KANA_MAP: Lazy<HashMap<char, DakuCnv>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        'ｶ',
        DakuCnv {
            nml: 'か',
            daku: 'が',
            handaku: None,
        },
    );
    m.insert(
        'ｷ',
        DakuCnv {
            nml: 'き',
            daku: 'ぎ',
            handaku: None,
        },
    );
    m.insert(
        'ｸ',
        DakuCnv {
            nml: 'く',
            daku: 'ぐ',
            handaku: None,
        },
    );
    m.insert(
        'ｹ',
        DakuCnv {
            nml: 'け',
            daku: 'げ',
            handaku: None,
        },
    );
    m.insert(
        'ｺ',
        DakuCnv {
            nml: 'こ',
            daku: 'ご',
            handaku: None,
        },
    );

    m.insert(
        'ｻ',
        DakuCnv {
            nml: 'さ',
            daku: 'ざ',
            handaku: None,
        },
    );
    m.insert(
        'ｼ',
        DakuCnv {
            nml: 'し',
            daku: 'じ',
            handaku: None,
        },
    );
    m.insert(
        'ｽ',
        DakuCnv {
            nml: 'す',
            daku: 'ず',
            handaku: None,
        },
    );
    m.insert(
        'ｾ',
        DakuCnv {
            nml: 'せ',
            daku: 'ぜ',
            handaku: None,
        },
    );
    m.insert(
        'ｿ',
        DakuCnv {
            nml: 'そ',
            daku: 'ぞ',
            handaku: None,
        },
    );

    m.insert(
        'ﾀ',
        DakuCnv {
            nml: 'た',
            daku: 'だ',
            handaku: None,
        },
    );
    m.insert(
        'ﾁ',
        DakuCnv {
            nml: 'ち',
            daku: 'ぢ',
            handaku: None,
        },
    );
    m.insert(
        'ﾂ',
        DakuCnv {
            nml: 'つ',
            daku: 'づ',
            handaku: None,
        },
    );
    m.insert(
        'ﾃ',
        DakuCnv {
            nml: 'て',
            daku: 'で',
            handaku: None,
        },
    );
    m.insert(
        'ﾄ',
        DakuCnv {
            nml: 'と',
            daku: 'ど',
            handaku: None,
        },
    );

    m.insert(
        'ﾊ',
        DakuCnv {
            nml: 'は',
            daku: 'ば',
            handaku: Some('ぱ'),
        },
    );
    m.insert(
        'ﾋ',
        DakuCnv {
            nml: 'ひ',
            daku: 'び',
            handaku: Some('ぴ'),
        },
    );
    m.insert(
        'ﾌ',
        DakuCnv {
            nml: 'ふ',
            daku: 'ぶ',
            handaku: Some('ぷ'),
        },
    );
    m.insert(
        'ﾍ',
        DakuCnv {
            nml: 'へ',
            daku: 'べ',
            handaku: Some('ぺ'),
        },
    );
    m.insert(
        'ﾎ',
        DakuCnv {
            nml: 'ほ',
            daku: 'ぼ',
            handaku: Some('ぽ'),
        },
    );

    m
});
