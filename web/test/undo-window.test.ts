import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { UNDO_WINDOW_MS, undoWindow } from "../src/undo-window.ts";

test("學到東西之後 3 秒叫核心讓撤銷紀錄失效；3 秒內不叫", () => {
	mock.timers.enable({ apis: ["setTimeout"] });
	try {
		let expired = 0;
		const window = undoWindow(() => expired++);
		window.restart();
		mock.timers.tick(UNDO_WINDOW_MS - 1);
		assert.equal(expired, 0);
		mock.timers.tick(1);
		assert.equal(expired, 1);
		mock.timers.tick(UNDO_WINDOW_MS * 2);
		assert.equal(expired, 1, "只叫一次");
	} finally {
		mock.timers.reset();
	}
});

test("又學到東西時從頭計時：3 秒從最後一次算", () => {
	mock.timers.enable({ apis: ["setTimeout"] });
	try {
		let expired = 0;
		const window = undoWindow(() => expired++);
		window.restart();
		mock.timers.tick(2000);
		window.restart();
		mock.timers.tick(2000);
		assert.equal(expired, 0);
		mock.timers.tick(1000);
		assert.equal(expired, 1);
	} finally {
		mock.timers.reset();
	}
});
