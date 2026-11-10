//! 網頁 e2e 匯入的小詞庫（`web/e2e/fixtures/e2e-lexicon/`，plan §4.6）：檔案進 git，這裡比對它和
//! 現在的資料包格式一致，格式改了會紅。`QIM_UPDATE_FIXTURES=1 cargo test -p qim-core --test
//! web_fixture` 重產。

use std::path::PathBuf;

use qim_core::{Lexicon, LexiconPack, MANIFEST_FILE, Manifest, Pack, PackKind, Word};

const ID: &str = "e2e-lexicon";

/// `anr` 只有一個詞「測試詞庫」：換上它時首選和內建詞庫的「門口」不同。
fn pack() -> LexiconPack {
    LexiconPack {
        id: ID.into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: "測試詞庫".into(),
                    frequency: 1000,
                }],
            )],
            [],
        ),
    }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/e2e/fixtures/e2e-lexicon")
}

#[test]
fn e2e_lexicon_fixture_is_current() {
    let bytes = pack().to_bytes().unwrap();
    let manifest = Manifest::new(PackKind::Lexicon, ID, &bytes, "1.0.0", "CC0-1.0").to_json();
    let pack_path = dir().join(format!("{ID}.pack"));
    let manifest_path = dir().join(MANIFEST_FILE);
    if std::env::var_os("QIM_UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(dir()).unwrap();
        std::fs::write(&pack_path, &bytes).unwrap();
        std::fs::write(&manifest_path, &manifest).unwrap();
    }
    let hint = "資料包格式或 manifest 改了：QIM_UPDATE_FIXTURES=1 cargo test -p qim-core --test web_fixture";
    assert_eq!(std::fs::read(&pack_path).ok(), Some(bytes), "{hint}");
    assert_eq!(
        std::fs::read_to_string(&manifest_path).ok(),
        Some(manifest),
        "{hint}"
    );
}
