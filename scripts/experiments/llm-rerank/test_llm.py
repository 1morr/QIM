"""llm.py 的測試：`python3 -m unittest test_llm`（在本目錄）。"""

import unittest

import llm

QUERY = {
    "split": "dev",
    "set": "hkcancor",
    "line": 7,
    "before": "我已經到咗你屋企樓下，",
    "input": "anr",
    "candidates": ["問", "門口", "閂", "門"],
    "paths": [
        {"text": "門口", "score": -1.0},
        {"text": "問口", "score": -2.0},
        {"text": "閂口", "score": -3.0},
    ],
}


class Choices(unittest.TestCase):
    def test_cand_keeps_the_first_page_in_core_order(self):
        self.assertEqual(llm.choices(QUERY, "cand", 10), ["問", "門口", "閂", "門"])

    def test_merge_puts_paths_first_then_unseen_candidates(self):
        self.assertEqual(
            llm.choices(QUERY, "merge", 10), ["門口", "問口", "閂口", "問", "閂", "門"]
        )

    def test_paths_keeps_only_the_whole_input_alternatives(self):
        self.assertEqual(llm.choices(QUERY, "paths", 10), ["門口", "問口", "閂口"])

    def test_choices_are_capped(self):
        self.assertEqual(llm.choices(QUERY, "merge", 4), ["門口", "問口", "閂口", "問"])


class Prompt(unittest.TestCase):
    def test_prompt_lists_numbered_choices_with_context_and_code(self):
        p = llm.build_prompt(QUERY, llm.STYLES["one"], "cand")
        self.assertEqual(p["split"], "dev")
        self.assertEqual((p["set"], p["line"]), ("hkcancor", 7))
        self.assertIn("我已經到咗你屋企樓下，", p["prompt"])
        self.assertIn("anr", p["prompt"])
        self.assertIn("1.問 2.門口 3.閂 4.門", p["prompt"])
        self.assertEqual(p["choices"], ["問", "門口", "閂", "門"])

    def test_empty_context_is_spelled_out(self):
        q = dict(QUERY, before="")
        p = llm.build_prompt(q, llm.STYLES["one"], "cand")
        self.assertIn("（無）", p["prompt"])

    def test_every_style_builds(self):
        for name, style in llm.STYLES.items():
            with self.subTest(name):
                p = llm.build_prompt(QUERY, style, "merge")
                self.assertTrue(p["instructions"])
                self.assertIn("1.門口", p["prompt"])


class ParseReply(unittest.TestCase):
    CHOICES = ["問", "門口", "閂", "門"]

    def test_bare_number(self):
        self.assertEqual(llm.parse_reply("2", self.CHOICES), ["門口"])

    def test_several_numbers_in_order_without_duplicates(self):
        self.assertEqual(llm.parse_reply("3, 2, 3", self.CHOICES), ["閂", "門口"])

    def test_out_of_range_and_noise_are_ignored(self):
        self.assertEqual(llm.parse_reply("序號：12 或 4。", self.CHOICES), ["門"])

    def test_fullwidth_digits(self):
        self.assertEqual(llm.parse_reply("２", self.CHOICES), ["門口"])

    def test_no_number(self):
        self.assertEqual(llm.parse_reply("門口", self.CHOICES), [])


class Orders(unittest.TestCase):
    def test_replies_become_ranked_text_keyed_by_set_and_line(self):
        prompts = [llm.build_prompt(QUERY, llm.STYLES["one"], "cand")]
        replies = [{"split": "dev", "set": "hkcancor", "line": 7, "reply": "2", "ms": 100}]
        self.assertEqual(
            llm.orders(prompts, replies),
            [{"split": "dev", "set": "hkcancor", "line": 7, "ranked": ["門口"]}],
        )

    def test_errors_and_late_replies_are_left_out(self):
        prompts = [llm.build_prompt(QUERY, llm.STYLES["one"], "cand")]
        replies = [
            {"split": "dev", "set": "hkcancor", "line": 7, "reply": "", "ms": 9, "error": "x"},
        ]
        self.assertEqual(llm.orders(prompts, replies), [])
        late = [{"split": "dev", "set": "hkcancor", "line": 7, "reply": "2", "ms": 3500}]
        self.assertEqual(llm.orders(prompts, late, deadline_ms=3000), [])
        self.assertEqual(len(llm.orders(prompts, late)), 1)


class Stats(unittest.TestCase):
    def test_percentiles_use_nearest_rank(self):
        self.assertEqual(llm.percentile([5, 1, 3, 2, 4], 50), 3)
        self.assertEqual(llm.percentile(list(range(1, 101)), 95), 95)

    def test_context_group_reads_the_set_suffix(self):
        self.assertEqual(llm.context_group("hkcancor"), "不給前文（維基、口語集）")
        self.assertEqual(llm.context_group("zh-wiki·前文0"), "前文 0")
        self.assertEqual(llm.context_group("yue-wiki·前文200"), "前文 200")

    def test_cost_per_thousand_uses_peak_prices(self):
        usage = [
            {"prompt_cache_hit_tokens": 100, "prompt_cache_miss_tokens": 200, "completion_tokens": 2}
        ] * 1000
        price = llm.PRICES["deepseek-flash"]
        expected = (100 * price["hit"] + 200 * price["miss"] + 2 * price["out"]) * 1000 / 1e6
        self.assertAlmostEqual(llm.cost_per_thousand(usage, "deepseek-flash"), expected)


class PersonalGuard(unittest.TestCase):
    def test_personal_set_is_refused_without_consent(self):
        with self.assertRaises(SystemExit):
            llm.check_personal([{"set": "personal"}], allow=False)
        llm.check_personal([{"set": "personal"}], allow=True)
        llm.check_personal([{"set": "hkcancor"}], allow=False)


class Subset(unittest.TestCase):
    def test_mcnemar_exact_two_sided(self):
        self.assertEqual(llm.mcnemar_exact(0, 0), 1.0)
        self.assertAlmostEqual(llm.mcnemar_exact(10, 0), 2 * 0.5**10)
        self.assertAlmostEqual(llm.mcnemar_exact(3, 5), llm.mcnemar_exact(5, 3))

    def test_subset_compares_only_the_prompted_lines(self):
        import tempfile
        from pathlib import Path

        head = "line\trank\tpath_rank\tcorrect\ttop2\tmodel_first\tcorrection_count\n"
        with tempfile.TemporaryDirectory() as d:
            a, b = Path(d, "a"), Path(d, "b")
            a.mkdir()
            b.mkdir()
            Path(a, "s.tsv").write_text(head + "1\t0\t0\t1\t0\t0\t0\n2\t0\t0\t1\t1\t1\t0\n3\t0\t0\t1\t0\t0\t0\n")
            Path(b, "s.tsv").write_text(head + "1\t0\t0\t1\t1\t1\t0\n2\t0\t0\t1\t0\t1\t0\n3\t0\t0\t1\t1\t0\t0\n")
            prompts = [{"split": "dev", "set": "s", "line": 1}, {"split": "dev", "set": "s", "line": 2}]
            r = llm.subset(prompts, a, b)
        self.assertEqual(r["total"], 2)
        self.assertEqual((r["top2_a"], r["top2_b"]), (1, 1))
        self.assertEqual((r["better"], r["worse"]), (1, 1))


class Cloud(unittest.TestCase):
    PROMPT = {"split": "dev", "set": "s", "line": 1, "instructions": "i", "prompt": "p"}

    def test_connection_errors_become_error_rows(self):
        import http.client
        import os
        from unittest import mock

        for exc in (http.client.RemoteDisconnected("x"), http.client.IncompleteRead(b""),
                    ConnectionResetError(), TimeoutError()):
            with self.subTest(type(exc).__name__), mock.patch.dict(os.environ, {"DEEPSEEK_API_KEY": "k"}), \
                    mock.patch("urllib.request.urlopen", side_effect=exc):
                r = llm.call("deepseek-flash", self.PROMPT, 1, 8)
            self.assertEqual(r["reply"], "")
            self.assertIn(type(exc).__name__, r["error"])

    def test_resume_keeps_answered_rows_and_resends_errors(self):
        prompts = [dict(self.PROMPT, line=i) for i in (1, 2, 3)]
        previous = [
            {"split": "dev", "set": "s", "line": 1, "reply": "2", "ms": 1},
            {"split": "dev", "set": "s", "line": 2, "reply": "", "ms": 1, "error": "HTTPError"},
        ]
        kept, todo = llm.resume(prompts, previous)
        self.assertEqual([r["line"] for r in kept], [1])
        self.assertEqual([p["line"] for p in todo], [2, 3])


if __name__ == "__main__":
    unittest.main()
