import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { restartTimer } from "../src/restart-timer.ts";

test("restart 之後過了呼叫端給的時限叫一次；時限內不叫", () => {
	mock.timers.enable({ apis: ["setTimeout"] });
	try {
		let fired = 0;
		const timer = restartTimer(() => fired++, 1100);
		timer.restart();
		mock.timers.tick(1099);
		assert.equal(fired, 0);
		mock.timers.tick(1);
		assert.equal(fired, 1);
		mock.timers.tick(1100 * 2);
		assert.equal(fired, 1, "只叫一次");
	} finally {
		mock.timers.reset();
	}
});

test("再 restart 時從頭計時：時限從最後一次算", () => {
	mock.timers.enable({ apis: ["setTimeout"] });
	try {
		let fired = 0;
		const timer = restartTimer(() => fired++, 3000);
		timer.restart();
		mock.timers.tick(2000);
		timer.restart();
		mock.timers.tick(2000);
		assert.equal(fired, 0);
		mock.timers.tick(1000);
		assert.equal(fired, 1);
	} finally {
		mock.timers.reset();
	}
});
