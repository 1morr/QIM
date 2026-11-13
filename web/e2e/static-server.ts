// e2e 扮演 GitHub release 的本機靜態伺服器（plan〈線上更新〉）：`node e2e/static-server.ts <目錄> <埠>`，
// 把目錄裡的檔照檔名送出，其他一律 404。Playwright 設定把它當第二個 webServer 起，Vite 開發伺服器的
// `/pack-updates/` 轉發到這裡（`QIM_PACK_UPDATE_URL`）。

import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { join } from "node:path";

const [root, port] = process.argv.slice(2);
if (!root || !port) {
	console.error("用法：node e2e/static-server.ts <目錄> <埠>");
	process.exit(2);
}

createServer(async (request, response) => {
	const name = (request.url ?? "").split("?")[0].replace(/^\//, "");
	if (request.method !== "GET" || !/^[A-Za-z0-9._-]+$/.test(name)) {
		response.writeHead(404).end();
		return;
	}
	try {
		const body = await readFile(join(root, name));
		const type = name.endsWith(".json")
			? "application/json"
			: "application/octet-stream";
		response.writeHead(200, { "Content-Type": type }).end(body);
	} catch {
		response.writeHead(404).end();
	}
}).listen(Number(port), "localhost");
