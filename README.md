# QIM

給速成熟手的全平台輸入法：速成能連打、首選準、會學使用者的用字，各平台體驗一致。QIM 是暫名。

## 現況

開發中，還不能當日常輸入法用。目前有一個網頁 prototype，可以試打速成連打、學習與觸控鍵盤；macOS、iOS、Android 的平台殼還沒開始。路線圖見 [`docs/plan.md`](docs/plan.md)〈里程碑任務〉，進度見 [`docs/progress.md`](docs/progress.md)。

## 試跑網頁 prototype

環境需求與完整指令見 [`docs/development.md`](docs/development.md)。簡略版：

```sh
npm ci --prefix web
cargo install wasm-bindgen-cli --version 0.2.130 --locked
cargo xtask data               # 建資料包，第一次要下載約 360 MB 的來源
cargo xtask web                # 編 WebAssembly 並複製資料包到網頁
npm run --prefix web dev
```

## 文件

- [`docs/design-brief.md`](docs/design-brief.md)：需求與決定
- [`docs/plan.md`](docs/plan.md)：架構與里程碑
- [`docs/development.md`](docs/development.md)：開發指令
- [`GLOSSARY.md`](GLOSSARY.md)、[`docs/adr/`](docs/adr/)：名詞表與架構決定

## 授權

程式碼以 [Apache-2.0](LICENSE) 授權。建置出的資料包各自帶上游來源的授權（`LICENSE`、`NOTICE`），來源與授權列在 [`data/sources.toml`](data/sources.toml)，授權全文在 [`data/licenses/`](data/licenses/)。
