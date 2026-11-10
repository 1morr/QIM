import { defineConfig, type Plugin } from "vite";

/**
 * 資料包線上更新的來源（plan §4.7）：GitHub 上 tag `packs` 那個 release 的檔案（不用 `latest`：App 的
 * release 也會發在同一個 repo）。環境變數 `QIM_PACK_UPDATE_URL` 可換（e2e 換成本機的靜態伺服器），
 * 結尾要有 `/`。
 */
const PACK_UPDATE_URL = "https://github.com/1morr/QIM/releases/download/packs/";
/** release 上的檔名：`index.json`、`<id>.pack` 這類，不收子路徑。 */
const ASSET_NAME = /^[A-Za-z0-9._-]+$/;

/**
 * 把網頁的 `/pack-updates/<檔名>` 轉給更新來源。GitHub 的 release 檔案（github.com 的轉址與
 * release-assets.githubusercontent.com）不帶 CORS 標頭，瀏覽器直接讀不了，所以經開發伺服器轉發，
 * 同 plan §3.1 雲端 API 的做法。只有開發伺服器有，`vite build` 出的靜態頁沒有線上更新。
 */
function packUpdates(): Plugin {
	return {
		name: "qim-pack-updates",
		configureServer(server) {
			const base = process.env.QIM_PACK_UPDATE_URL ?? PACK_UPDATE_URL;
			server.middlewares.use("/pack-updates/", async (request, response) => {
				const name = (request.url ?? "").replace(/^\//, "").split("?")[0];
				if (request.method !== "GET" || !ASSET_NAME.test(name)) {
					response.statusCode = 404;
					response.end();
					return;
				}
				try {
					// Node 的 fetch 會跟著 GitHub 的轉址走。
					const upstream = await fetch(new URL(name, base));
					response.statusCode = upstream.status;
					response.setHeader("Cache-Control", "no-store");
					const type = upstream.headers.get("content-type");
					if (type) {
						response.setHeader("Content-Type", type);
					}
					response.end(Buffer.from(await upstream.arrayBuffer()));
				} catch (error) {
					response.statusCode = 502;
					response.end(`連不上更新來源 ${base}：${String(error)}`);
				}
			});
		},
	};
}

export default defineConfig({
	plugins: [packUpdates()],
});
