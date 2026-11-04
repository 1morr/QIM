use qim_databuild::hkcancor::utterances;

/// HKCanCor `utf8/FC-107_v2` 的開頭（fcbond/hkcancor `39aeadf`）。
const TRANSCRIPT: &str = "<info>
\t1-TN-107
\tINFO-END
</info>
\t<sent>
\t\t<sent_head>
\t\t\tA:
\t\t</sent_head>
\t\t<sent_tag>
\t\t\t陣間/t/zan6gaan1/
\t\t\t放學/v/fong3hok6/
\t\t\t去/v/heoi3/
\t\t\t邊/r/bin1/
\t\t\t啊/y/aa3/
\t\t\t？/w/VQ6/
\t\t</sent_tag>
\t\t<sent_tran>

\t\t</sent_tran>
\t</sent>
\t<sent>
\t\t<sent_head>
\t\t\tB:
\t\t</sent_head>
\t\t<sent_tag>
\t\t\t冇/v1/mou5/
\t\t\t啊/y1/aa3/
\t\t\t，/w/VQ2/
\t\t\tHong Kong U/n/hoeng1gong2ju1/
\t\t\t-/w/VQ3/
\t\t</sent_tag>
\t</sent>
";

#[test]
fn each_sentence_becomes_one_utterance_of_its_words() {
    assert_eq!(
        utterances(TRANSCRIPT),
        ["陣間放學去邊啊？", "冇啊，Hong Kong U-"]
    );
}

#[test]
fn transcript_without_sentences_has_no_utterances() {
    assert!(utterances("<info>\n</info>\n").is_empty());
}
