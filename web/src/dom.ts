/** 取 `index.html` 裡一定有的元素；少了就是頁面與程式對不上，直接丟錯。 */
export function element<T extends HTMLElement>(selector: string): T {
	const found = document.querySelector<T>(selector);
	if (!found) {
		throw new Error(`index.html 少了 ${selector}`);
	}
	return found;
}
