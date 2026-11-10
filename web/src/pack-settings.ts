// 設定區的資料包（plan §3.1、§4.6、§4.7）：列出名稱、版本、大小、授權，切換每種用的那份，刪除匯入的，
// 從檔案匯入，線上更新的開關與「檢查更新」。換了之後由 `apply` 交給核心；核心不收時退回原本的樣子
// （`PackLibrary` 的匯入、刪除也是）。

import { element, message } from "./dom.ts";
import { kindName, type PackOrigin } from "./pack-choice.ts";
import {
	setUpdatesEnabled,
	type UpdateResult,
	updatePacks,
	updatesEnabled,
} from "./pack-update.ts";
import type { ListedPack, PackLibrary } from "./packs.ts";

const ORIGIN_NAMES: Readonly<Record<PackOrigin, string>> = {
	builtin: "內建",
	imported: "匯入",
	downloaded: "線上更新",
};

function sizeText(bytes: number): string {
	if (bytes >= 1024 * 1024) {
		return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
	}
	return bytes >= 1024 ? `${Math.round(bytes / 1024)} KB` : `${bytes} B`;
}

/** 線上更新的結果；`manual` 是按了「檢查更新」，沒有新版時也說。沒有要說的回傳 `undefined`。 */
function updateText(result: UpdateResult, manual: boolean): string | undefined {
	const parts = [
		...result.updated.map((u) => `已更新 ${u.id} ${u.from} → ${u.to}`),
		...result.rejected.map(
			(r) => `${r.id} ${r.version} 沒換上，照用現有的：${r.reason}`,
		),
	];
	if (parts.length > 0) {
		return parts.join("；");
	}
	return manual ? "資料包都是最新的" : undefined;
}

/**
 * 接上資料包區。`apply` 把目前選的資料包交給核心，不收時丟錯；`afterChange` 在使用者的每次動作之後
 * 呼叫（網頁把焦點還給輸入框）。線上更新開著時，接上後就在背景檢查一次。
 */
export function mountPackSettings(
	library: PackLibrary,
	apply: () => void,
	afterChange: () => void,
): void {
	const body = element<HTMLTableSectionElement>("#pack-rows");
	const file = element<HTMLInputElement>("#import-pack");
	const status = element<HTMLElement>("#pack-status");

	const activeOf = (kind: string): string | undefined =>
		library.list().find((p) => p.kind === kind && p.active)?.id;

	const row = (pack: ListedPack, sameKind: number): HTMLTableRowElement => {
		const tr = document.createElement("tr");
		tr.dataset.id = pack.id;
		const cell = (text: string) => {
			const td = document.createElement("td");
			td.textContent = text;
			return td;
		};
		const use = document.createElement("td");
		const radio = document.createElement("input");
		radio.type = "radio";
		radio.name = `pack-${pack.kind}`;
		radio.checked = pack.active;
		radio.disabled = sameKind < 2;
		radio.setAttribute("aria-label", `使用 ${pack.id}`);
		radio.addEventListener("change", () => {
			const before = activeOf(pack.kind);
			library.choose(pack.kind, pack.id);
			try {
				apply();
				status.textContent = `${kindName(pack.kind)}改用 ${pack.id}`;
			} catch (error) {
				// 核心不收時沒換掉原本的 Engine，選擇退回去就好。
				if (before) {
					library.choose(pack.kind, before);
				}
				status.textContent = `換不了，照用原本的：${message(error)}`;
			}
			render();
			afterChange();
		});
		use.append(radio);
		const actions = document.createElement("td");
		if (pack.origin !== "builtin") {
			const remove = document.createElement("button");
			remove.type = "button";
			remove.textContent = "刪除";
			remove.setAttribute("aria-label", `刪除 ${pack.id}`);
			remove.addEventListener("click", async () => {
				try {
					await library.remove(pack.id, apply);
					status.textContent = `已刪除 ${pack.id}`;
				} catch (error) {
					status.textContent = `刪不掉：${message(error)}`;
				}
				render();
				afterChange();
			});
			actions.append(remove);
		}
		tr.append(
			use,
			cell(kindName(pack.kind)),
			cell(pack.id),
			cell(pack.version),
			cell(sizeText(pack.size)),
			cell(pack.license),
			cell(ORIGIN_NAMES[pack.origin]),
			actions,
		);
		return tr;
	};

	const render = (): void => {
		const listed = library.list();
		body.replaceChildren(
			...listed.map((pack) =>
				row(pack, listed.filter((p) => p.kind === pack.kind).length),
			),
		);
	};

	const updates = element<HTMLInputElement>("#pack-updates");
	const check = element<HTMLButtonElement>("#check-pack-updates");
	let checking = false;
	const syncCheck = (): void => {
		check.disabled = checking || !updates.checked;
	};
	/** 檢查並換上新版。背景的（`manual` 是 `false`）不搶焦點，沒有新版時也不說。 */
	const runUpdates = async (manual: boolean): Promise<void> => {
		checking = true;
		syncCheck();
		try {
			const text = updateText(await updatePacks(library, apply), manual);
			if (text) {
				status.textContent = text;
			}
		} catch (error) {
			status.textContent = `檢查更新失敗，照用現有的資料包：${message(error)}`;
		}
		checking = false;
		syncCheck();
		render();
		if (manual) {
			afterChange();
		}
	};
	updates.checked = updatesEnabled();
	updates.addEventListener("change", () => {
		setUpdatesEnabled(updates.checked);
		syncCheck();
		afterChange();
	});
	check.addEventListener("click", () => {
		void runUpdates(true);
	});
	syncCheck();

	file.addEventListener("change", async () => {
		const chosen = [...(file.files ?? [])];
		file.value = "";
		if (chosen.length === 0) {
			return;
		}
		try {
			const entry = await library.import(chosen, apply);
			status.textContent = `已匯入 ${entry.id} ${entry.version}，${kindName(entry.kind)}改用它`;
		} catch (error) {
			status.textContent = `匯入失敗，資料包沒動：${message(error)}`;
		}
		render();
		afterChange();
	});

	render();
	if (updates.checked) {
		void runUpdates(false);
	}
}
