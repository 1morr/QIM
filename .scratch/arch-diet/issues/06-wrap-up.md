# 06: §8 歸位、改引用、刪掉 plan

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05
讀: `.scratch/arch-diet/spec.md`；plan 剩下的全部；`CLAUDE.md`；`docs/agents/`；roadmap；brief；M4 還沒做的票

## What to build

把 plan §8〈風險與回寫點〉歸位，確認 plan 每一節都只剩標題與指向，然後刪掉 `docs/plan.md`。改引用：`CLAUDE.md`、`docs/agents/milestone.md`、`docs/agents/ticket-template.md`、`docs/agents/issue-tracker.md`、roadmap 的「拆票前讀」、brief、ADR、M4 還沒做的票與 spec、程式碼註解裡的 `plan〈…〉`，都改成 `architecture〈節名〉` 或實際的檔。最後把使用者試的清單給使用者，`docs/progress.md` 的下一步寫 `等使用者：讀 ARCHITECTURE.md`。

## 驗收

- [ ] `docs/plan.md` 已刪；`ARCHITECTURE.md` 不超過 15,000 字（`python3 scripts/check_docs.py` 會擋）。
- [ ] 除了已封存的里程碑，`grep -rn "plan §\|docs/plan.md\|plan〈"` 沒有結果。
- [ ] 使用者讀過 `ARCHITECTURE.md` 並回覆可以（回覆記在 Comments）。
- [ ] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
