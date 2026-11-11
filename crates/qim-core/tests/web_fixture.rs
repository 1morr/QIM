//! 網頁 e2e 用的小詞庫（plan §4.6、§4.7）：檔案進 git，這裡比對它們和現在的資料包、manifest 與
//! 更新索引的格式一致，格式改了會紅。`QIM_UPDATE_FIXTURES=1 cargo test -p qim-core --test
//! web_fixture` 重產。
//!
//! - `e2e-lexicon/`：從檔案匯入的詞庫。
//! - `e2e-update-lexicon/`：線上更新前先匯入的舊版（1.0.0）。
//! - `quick-lexicon-1.0.1/`、`quick-lexicon-1.0.2/`：和內建詞庫同 id 的兩版，e2e 攔請求換上，驗下載版與
//!   內建版取版本較高的那份（plan §4.7）。
//!
//! 扮演 GitHub release 的 `pack-release/` 由 `qim-databuild` 的 `tests/release.rs` 照 `cargo xtask
//! pack-release` 的路寫出。

use std::path::{Path, PathBuf};

use qim_core::{Lexicon, LexiconPack, MANIFEST_FILE, Manifest, Pack, PackKind, Word};

/// `anr` 只有一個詞 `word`：換上它時首選和內建詞庫的「門口」不同。
fn lexicon(id: &str, word: &str) -> Vec<u8> {
    LexiconPack {
        id: id.into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: word.into(),
                    frequency: 1000,
                }],
            )],
            [],
        ),
    }
    .to_bytes()
    .unwrap()
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/e2e/fixtures")
}

/// 照 `QIM_UPDATE_FIXTURES` 寫出，或比對檔案和 `contents` 相同。
fn check(path: &Path, contents: &[u8]) {
    if std::env::var_os("QIM_UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    assert_eq!(
        std::fs::read(path).ok().as_deref(),
        Some(contents),
        "{} 和現在的格式不同：QIM_UPDATE_FIXTURES=1 cargo test -p qim-core --test web_fixture",
        path.display()
    );
}

/// 一份匯入用的資料包目錄：`<id>/manifest.json` 與 `<id>/<id>.pack`。
fn check_pack_dir(id: &str, word: &str, version: &str) {
    check_pack_in(id, id, word, version);
}

/// 同上，目錄名是 `dir`。
fn check_pack_in(dir: &str, id: &str, word: &str, version: &str) {
    let bytes = lexicon(id, word);
    let manifest = Manifest::new(PackKind::Lexicon, id, &bytes, version, "CC0-1.0");
    let dir = fixtures().join(dir);
    check(&dir.join(manifest.file_name()), &bytes);
    check(&dir.join(MANIFEST_FILE), manifest.to_json().as_bytes());
}

#[test]
fn e2e_lexicon_fixture_is_current() {
    check_pack_dir("e2e-lexicon", "測試詞庫", "1.0.0");
}

#[test]
fn update_lexicon_fixture_is_current() {
    check_pack_dir("e2e-update-lexicon", "舊版詞庫", "1.0.0");
}

#[test]
fn builtin_lexicon_versions_are_current() {
    check_pack_in(
        "quick-lexicon-1.0.1",
        "quick-lexicon",
        "下載版詞庫",
        "1.0.1",
    );
    check_pack_in(
        "quick-lexicon-1.0.2",
        "quick-lexicon",
        "內建新版詞庫",
        "1.0.2",
    );
}
