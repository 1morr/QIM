import { defineConfig, devices } from "@playwright/test";

// 專用埠：不重用開著的開發伺服器，避免測到舊的綁定或資料包。
const port = 5179;
const url = `http://localhost:${port}`;

export default defineConfig({
	testDir: "e2e",
	forbidOnly: true,
	use: { baseURL: url },
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
	webServer: {
		command: `npm run dev -- --port ${port} --strictPort`,
		url,
		reuseExistingServer: false,
	},
});
