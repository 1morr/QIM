// 裝置 id（plan〈學習與排位〉）：匯出檔頭帶著它，跨裝置合併靠它分各台的次數。

/** 產生裝置 id 用到的 `crypto`：`randomUUID` 只在安全環境（HTTPS、localhost）才有，區網 http 沒有。 */
export interface DeviceIdCrypto {
	getRandomValues(array: Uint8Array<ArrayBuffer>): Uint8Array<ArrayBuffer>;
	randomUUID?(): string;
}

/** 新的裝置 id：RFC 4122 第 4 版 UUID。有 `randomUUID` 用它，沒有就用 `getRandomValues`（非安全環境也有）組。 */
export function newDeviceId(source: DeviceIdCrypto = crypto): string {
	if (source.randomUUID) {
		return source.randomUUID();
	}
	const bytes = source.getRandomValues(new Uint8Array(16));
	// RFC 4122 §4.4：第 7 個位元組的高 4 位是版本 4，第 9 個位元組的高 2 位是變體 10。
	bytes[6] = (bytes[6] & 0x0f) | 0x40;
	bytes[8] = (bytes[8] & 0x3f) | 0x80;
	const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0"));
	return [
		hex.slice(0, 4),
		hex.slice(4, 6),
		hex.slice(6, 8),
		hex.slice(8, 10),
		hex.slice(10, 16),
	]
		.map((group) => group.join(""))
		.join("-");
}
