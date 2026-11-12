import assert from "node:assert/strict";
import { test } from "node:test";
import { localToday } from "../src/today.ts";

test("當地的日曆日寫成 YYYY-MM-DD，月日補零", () => {
	assert.equal(localToday(new Date(2026, 10, 12, 9, 30)), "2026-11-12");
	assert.equal(localToday(new Date(2027, 0, 5)), "2027-01-05");
	assert.equal(localToday(new Date(999, 1, 3)), "0999-02-03");
});

test("用當地時間，不是 UTC：午夜前一刻還是當天", () => {
	assert.equal(localToday(new Date(2026, 11, 31, 23, 59, 59)), "2026-12-31");
	assert.equal(localToday(new Date(2027, 0, 1, 0, 0, 0)), "2027-01-01");
});
