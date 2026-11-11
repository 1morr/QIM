import { createHash } from "node:crypto";
import { defineConfig, devices } from "@playwright/test";

/**
 * e2e 的埠：`QIM_E2E_PORT` 有設就用它，否則由這份設定所在目錄的絕對路徑算出 20000–29999 之間的埠。
 * 每個 worktree 固定一個埠，並行的 worktree 不互搶；不隨機挑，因為每個 worker 行程都會重新載入設定，
 * 隨機的埠會和主行程對不上。
 */
function e2ePort(): number {
	const fromEnv = process.env.QIM_E2E_PORT;
	if (fromEnv) {
		return Number(fromEnv);
	}
	const digest = createHash("sha256")
		.update(import.meta.dirname)
		.digest();
	return 20000 + (digest.readUInt32BE(0) % 10000);
}

const port = e2ePort();
const url = `http://localhost:${port}`;
/**
 * 扮演 GitHub release 的本機靜態伺服器（`e2e/static-server.ts`）放 `e2e/fixtures/pack-release/`；埠是
 * e2e 的埠加 10000，同樣每個 worktree 固定一個。開發伺服器的 `/pack-updates/` 轉發到這裡。
 */
const releasePort = port + 10000;

export default defineConfig({
	testDir: "e2e",
	forbidOnly: true,
	use: { baseURL: url },
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
	webServer: [
		{
			command: `node e2e/static-server.ts e2e/fixtures/pack-release ${releasePort}`,
			url: `http://localhost:${releasePort}/index.json`,
			reuseExistingServer: false,
		},
		{
			command: `npm run dev -- --port ${port} --strictPort`,
			url,
			env: { QIM_PACK_UPDATE_URL: `http://localhost:${releasePort}/` },
			// 埠已被佔用時直接失敗，不接手別人起的伺服器：那可能是別的 worktree，跑到的是別張票的程式碼。
			reuseExistingServer: false,
		},
	],
});
