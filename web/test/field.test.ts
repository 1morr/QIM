import assert from "node:assert/strict";
import { test } from "node:test";
import { fieldKind } from "../src/field.ts";

test("type 對到欄位類型（Chrome 轉給 Android 的 inputType，事實 P24）", () => {
	assert.equal(fieldKind("email", null), "email");
	assert.equal(fieldKind("url", null), "url");
	assert.equal(fieldKind("number", null), "number");
	assert.equal(fieldKind("tel", null), "number");
	assert.equal(fieldKind("password", null), "password");
	assert.equal(fieldKind("text", null), "text");
	assert.equal(fieldKind("search", null), "text");
	assert.equal(fieldKind(null, null), "text");
});

test("inputmode 蓋過 type（Chrome 的 inputmode=numeric 是 NUMBER）", () => {
	assert.equal(fieldKind("text", "numeric"), "number");
	assert.equal(fieldKind(null, "decimal"), "number");
	assert.equal(fieldKind(null, "tel"), "number");
	assert.equal(fieldKind("text", "email"), "email");
	assert.equal(fieldKind("text", "url"), "url");
	assert.equal(fieldKind("email", "text"), "email");
	assert.equal(fieldKind("url", "search"), "url");
});

test("密碼欄不論 inputmode 都是密碼", () => {
	assert.equal(fieldKind("password", "numeric"), "password");
});

test("不認得的值照一般欄", () => {
	assert.equal(fieldKind("color", "none"), "text");
	assert.equal(fieldKind("EMAIL", null), "email");
});
