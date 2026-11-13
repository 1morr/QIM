//! 打分參數的驗證（plan〈排序：同一把尺〉）：方案檔、`bench --set` 與 `Engine::new` 共用 `Ranking::check`。

use qim_core::{CodeTable, Engine, Packs, Ranking, RankingError, Scheme, SchemePack};

#[test]
fn the_default_and_a_beam_of_one_pass() {
    assert_eq!(Ranking::default().check(), Ok(()));
    let ranking = Ranking {
        beam: 1,
        sentences: 0,
        word_bonus: -3.0,
        ..Ranking::default()
    };
    assert_eq!(ranking.check(), Ok(()));
}

#[test]
fn a_beam_of_zero_is_rejected_and_named() {
    let error = Ranking {
        beam: 0,
        ..Ranking::default()
    }
    .check()
    .unwrap_err();
    assert_eq!(error, RankingError::ZeroBeam);
    assert!(error.to_string().contains("beam"), "{error}");
}

#[test]
fn scores_that_are_not_finite_are_rejected_with_the_field_and_value() {
    let nan = Ranking {
        word_bonus: f64::NAN,
        ..Ranking::default()
    }
    .check()
    .unwrap_err();
    let message = nan.to_string();
    assert!(
        message.contains("word_bonus") && message.contains("NaN"),
        "{message}"
    );

    let inf = Ranking {
        segment_penalty: f64::NEG_INFINITY,
        ..Ranking::default()
    }
    .check()
    .unwrap_err();
    let message = inf.to_string();
    assert!(
        message.contains("segment_penalty") && message.contains("inf"),
        "{message}"
    );
}

/// 資料包不一定由 `qim-databuild` 建（殼可以從檔案匯入），引擎載入時也擋。
#[test]
fn an_engine_refuses_a_scheme_with_a_bad_ranking() {
    let pack = SchemePack {
        scheme: Scheme {
            id: "quick".into(),
            name: "速成".into(),
            version: "1".into(),
            alphabet: "a".into(),
            max_code_length: 1,
            wildcard: None,
            radicals: [('a', "日".to_string())].into(),
            packs: Packs::default(),
            ranking: Ranking {
                beam: 0,
                ..Ranking::default()
            },
        },
        code_table: CodeTable::from_entries(Vec::new()),
    };
    let Err(error) = Engine::new(pack, None, None) else {
        panic!("beam 0 要被拒");
    };
    assert!(error.to_string().contains("beam"), "{error}");
}
