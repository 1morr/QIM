# gates：補齊閘門

不是里程碑，規格就是本檔（使用者 2026-11-15 同意開成票）。1morr 的 project-init 範本要求 `docs/development.md`〈閘門〉的八道閘門每道都有指令或設定檔；QIM 缺密鑰掃描、依賴漏洞、依賴更新三道。不改產品行為。

## 目標

〈閘門〉表沒有「待定」：三道缺的閘門各有指令或設定檔，接進 `cargo xtask check`、pre-commit 或 CI，flow-sync 換上範本的 `scripts/check_docs.py` 時這張表直接過。

## 範圍

### 做

- 密鑰掃描：擋 token、密碼被 commit，接 pre-commit 與 CI。
- 依賴漏洞：Rust crate（`Cargo.lock`）與 `web/` 的 npm 套件（`web/package-lock.json`）都掃，接 CI。
- 依賴更新：Dependabot 或 Renovate 的設定檔，涵蓋 cargo、npm、GitHub Actions。
- `docs/development.md` 的〈環境需求〉〈閘門〉〈全部檢查〉與 `xtask/src/check.rs` 同步改。

### 不做

- 不升級現有依賴、不修掃出來的舊漏洞：掃出來的各記一條到票的 Comments，有修的必要再開票。
- 不改 `deps` 步驟（依賴規則的白名單）。

## 共同規則

1. 工具照 Rust 與 npm 現在的主流選，先查證（context7、官方文件）再定，查到的寫進 `docs/research/` 的事實檔；在兩個工具之間選了一個，記一行 `決定：`。
2. pre-commit 照舊幾秒內跑完、不連網路；要連網路的掃描只進完整的 `cargo xtask check` 與 CI。
3. 新步驟加進 `xtask/src/check.rs` 時，同輪改〈全部檢查〉的表，`cargo test -p xtask` 比對兩邊。
4. 每張票收尾跑 `cargo xtask check`。

## 驗收

### Agent 驗

- `docs/development.md`〈閘門〉每一列都有行內程式碼的指令或設定檔，沒有「待定」。
- 造一個假的 token 檔 `git add` 後 commit，pre-commit 擋下；拿掉後放行。輸出貼在票上。
- `cargo xtask check` 與 CI 綠燈。

### 使用者試

- 無：只加開發閘門，沒有使用者看得到的產品改動。

## 票序

01。
