//! `docs` 步驟：`docs/progress.md`、里程碑 spec 與收尾票的規則（`docs/plan.md` §6）。

use std::path::Path;

const PROGRESS: &str = "docs/progress.md";

/// 里程碑 spec 與票所在的目錄。
const SCRATCH: &str = ".scratch";

const MILESTONES: &str = "里程碑與閱讀指引";
const SESSIONS: &str = "Session 紀錄";
const DEVIATIONS: &str = "偏差與決定";

/// progress 的 `##` 段，依序各一次。
const PROGRESS_SECTIONS: [&str; 4] = ["寫法與封存", MILESTONES, SESSIONS, DEVIATIONS];

/// 里程碑 spec 必備的 `##` 段；標題可以在後面加全形括號的說明，例如「範圍（做 / 不做）」。
const SPEC_SECTIONS: [&str; 5] = ["目標", "範圍", "共同規則", "驗收", "票序"];

/// session 紀錄每列、偏差每條的整行上限（Unicode 字元）。
const MAX_CHARS: usize = 200;

/// 「下一步」可以等使用者決定的開頭。
const WAITING: &str = "等使用者：";

/// 跑 [`run`] 並把違規印到終端機；回傳是否通過。
pub fn report(root: &Path) -> bool {
    match run(root) {
        Ok(violations) => {
            for violation in &violations {
                eprintln!("{violation}");
            }
            violations.is_empty()
        }
        Err(error) => {
            eprintln!("{error}");
            false
        }
    }
}

/// 檢查 `root` 的 progress 與 `.scratch/`，回傳每一條違規的說明；空的就是通過。
pub fn run(root: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(root.join(PROGRESS))
        .map_err(|error| format!("讀不到 {PROGRESS}：{error}"))?;
    let sections = sections(&text);
    let mut violations = Vec::new();
    check_section_order(&sections, &mut violations);
    let milestones = section(&sections, MILESTONES)
        .map(|section| milestones(section, &mut violations))
        .unwrap_or_default();
    let active = active_milestone(&milestones, &mut violations);
    if let Some(sessions) = section(&sections, SESSIONS) {
        check_one_table(sessions, &mut violations);
        check_lengths(sessions, &mut violations);
        check_session_rows(sessions, active, &mut violations);
    }
    if let Some(deviations) = section(&sections, DEVIATIONS) {
        check_lengths(deviations, &mut violations);
        check_deviations(deviations, active, &mut violations);
    }
    check_scratch(root, &milestones, &mut violations)?;
    Ok(violations)
}

/// 一行內容與它的行號（從 1 起算）。
#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    text: &'a str,
}

/// 一個 `##` 段：標題、標題所在行、到下一個 `##` 之前的內容。
struct Section<'a> {
    title: &'a str,
    line: usize,
    body: Vec<Line<'a>>,
}

/// 切出 `##` 段；第一個 `##` 之前的內容不屬於任何段。程式碼區塊內的 `##` 不算標題。
fn sections(text: &str) -> Vec<Section<'_>> {
    let mut sections: Vec<Section<'_>> = Vec::new();
    // 開著的程式碼區塊用的圍欄字元；只有同一種圍欄才關得掉它。
    let mut fence: Option<char> = None;
    for (index, text) in text.lines().enumerate() {
        let number = index + 1;
        let trimmed = text.trim_start();
        for marker in ['`', '~'] {
            if trimmed.starts_with(&marker.to_string().repeat(3)) {
                fence = match fence {
                    None => Some(marker),
                    Some(open) if open == marker => None,
                    open => open,
                };
            }
        }
        match text.strip_prefix("## ") {
            Some(title) if fence.is_none() => sections.push(Section {
                title: title.trim(),
                line: number,
                body: Vec::new(),
            }),
            _ => {
                if let Some(section) = sections.last_mut() {
                    section.body.push(Line { number, text });
                }
            }
        }
    }
    sections
}

/// 第一個標題是 `title` 的段；重複的段由 [`check_section_order`] 回報，其他規則只看第一個。
fn section<'s, 'a>(sections: &'s [Section<'a>], title: &str) -> Option<&'s Section<'a>> {
    sections.iter().find(|section| section.title == title)
}

fn check_section_order(sections: &[Section<'_>], violations: &mut Vec<String>) {
    let order = PROGRESS_SECTIONS
        .map(|title| format!("〈{title}〉"))
        .concat();
    let mut seen: Vec<usize> = Vec::new();
    for section in sections {
        let title = section.title;
        let line = section.line;
        let Some(index) = PROGRESS_SECTIONS.iter().position(|known| *known == title) else {
            violations.push(format!(
                "{PROGRESS}:{line}: 多出〈{title}〉段：progress 的 ## 段只有{order}"
            ));
            continue;
        };
        if seen.contains(&index) {
            violations.push(format!(
                "{PROGRESS}:{line}: 〈{title}〉段重複：## 段各一次（union 合併會安靜地抄出重複段，刪掉這段並把其中新的列併回第一段）"
            ));
        } else if seen.iter().any(|&earlier| earlier > index) {
            violations.push(format!(
                "{PROGRESS}:{line}: 〈{title}〉段順序錯：## 段依序是{order}"
            ));
        }
        seen.push(index);
    }
    for (index, title) in PROGRESS_SECTIONS.iter().enumerate() {
        if !seen.contains(&index) {
            violations.push(format!("{PROGRESS}: 缺〈{title}〉段：## 段依序是{order}"));
        }
    }
}

fn is_table_row(text: &str) -> bool {
    text.trim_start().starts_with('|')
}

/// 段內只有一張表：第一列到最後一列之間每行都是表格列，表外沒有其他文字。
fn check_one_table(section: &Section<'_>, violations: &mut Vec<String>) {
    let title = section.title;
    let rows = || section.body.iter().filter(|line| is_table_row(line.text));
    let (Some(first), Some(last)) = (rows().next(), rows().next_back()) else {
        violations.push(format!(
            "{PROGRESS}:{}: 〈{title}〉要是一整張表，現在沒有表",
            section.line
        ));
        return;
    };
    for line in &section.body {
        if is_table_row(line.text) {
            continue;
        }
        let number = line.number;
        if (first.number..last.number).contains(&number) {
            violations.push(format!(
                "{PROGRESS}:{number}: 〈{title}〉要是一整張表，表格在這一行斷開"
            ));
        } else if !line.text.trim().is_empty() {
            violations.push(format!(
                "{PROGRESS}:{number}: 〈{title}〉要是一整張表，表外不放文字"
            ));
        }
    }
}

/// 段內每一行不超過 [`MAX_CHARS`] 個 Unicode 字元。
fn check_lengths(section: &Section<'_>, violations: &mut Vec<String>) {
    for line in &section.body {
        let chars = line.text.chars().count();
        if chars > MAX_CHARS {
            violations.push(format!(
                "{PROGRESS}:{}: 整行 {chars} 字元，超過 {MAX_CHARS}：細節移到票的 ## Comments，這裡只留一句指向它",
                line.number
            ));
        }
    }
}

/// 段內第一張表的表頭與資料列（略過分隔列）。
struct Table<'a> {
    header: Line<'a>,
    rows: Vec<(usize, Vec<&'a str>)>,
}

impl Table<'_> {
    fn column(&self, name: &str) -> Option<usize> {
        cells(self.header.text)
            .iter()
            .position(|cell| *cell == name)
    }
}

fn table<'a>(section: &Section<'a>) -> Option<Table<'a>> {
    let mut lines = section
        .body
        .iter()
        .skip_while(|line| !is_table_row(line.text))
        .take_while(|line| is_table_row(line.text));
    let header = *lines.next()?;
    let is_separator = |cells: &[&str]| {
        cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| matches!(c, '-' | ':')))
    };
    let rows = lines
        .map(|line| (line.number, cells(line.text)))
        .filter(|(_, cells)| !is_separator(cells))
        .collect();
    Some(Table { header, rows })
}

/// 表格列的各欄，去掉前後空白；`\|` 是欄內的直線，不分欄。
pub(crate) fn cells(text: &str) -> Vec<&str> {
    let text = text.trim();
    let text = text.strip_prefix('|').unwrap_or(text);
    let text = text.strip_suffix('|').unwrap_or(text);
    let mut cells = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (index, c) in text.char_indices() {
        match c {
            '|' if !escaped => {
                cells.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
        escaped = c == '\\' && !escaped;
    }
    cells.push(text[start..].trim());
    cells
}

/// 里程碑表的一列。
struct Milestone<'a> {
    id: &'a str,
    note: &'a str,
}

impl Milestone<'_> {
    fn in_progress(&self) -> bool {
        self.note.starts_with("進行中")
    }

    fn not_started(&self) -> bool {
        self.note.starts_with("未開始")
    }
}

/// 讀〈里程碑與閱讀指引〉的表：依表頭找「里程碑」與「備註」欄。
fn milestones<'a>(section: &Section<'a>, violations: &mut Vec<String>) -> Vec<Milestone<'a>> {
    let Some(table) = table(section) else {
        violations.push(format!(
            "{PROGRESS}:{}: 〈{MILESTONES}〉缺里程碑表",
            section.line
        ));
        return Vec::new();
    };
    let (Some(id), Some(note)) = (table.column("里程碑"), table.column("備註")) else {
        violations.push(format!(
            "{PROGRESS}:{}: 里程碑表的表頭要有「里程碑」與「備註」欄",
            table.header.number
        ));
        return Vec::new();
    };
    table
        .rows
        .iter()
        .filter_map(|(_, cells)| {
            Some(Milestone {
                id: cells.get(id)?,
                note: cells.get(note)?,
            })
        })
        .collect()
}

/// 備註以「進行中」開頭的那一個里程碑；不是剛好一個就回報並回傳 `None`。
/// 封存一個里程碑與把下一個標成進行中在同一個 commit（progress〈寫法與封存〉），所以不會有零個的空檔。
fn active_milestone<'a>(
    milestones: &[Milestone<'a>],
    violations: &mut Vec<String>,
) -> Option<&'a str> {
    let active: Vec<&str> = milestones
        .iter()
        .filter(|milestone| milestone.in_progress())
        .map(|milestone| milestone.id)
        .collect();
    match active.as_slice() {
        [id] => Some(id),
        _ => {
            if !milestones.is_empty() {
                violations.push(format!(
                    "{PROGRESS}: 里程碑表要剛好一個備註以「進行中」開頭的里程碑，現在有 {} 個",
                    active.len()
                ));
            }
            None
        }
    }
}

/// 條目開頭標的里程碑要是進行中的那一個；`active` 是 `None` 時已另外回報，不再逐條報。
fn check_marker(number: usize, marker: &str, active: Option<&str>, violations: &mut Vec<String>) {
    if let Some(active) = active
        && marker != active
    {
        violations.push(format!(
            "{PROGRESS}:{number}: 標 {marker}，但進行中的里程碑是 {active}：本檔只記進行中的里程碑，其他的封存到 docs/archive/"
        ));
    }
}

/// 字串開頭的里程碑標記（`M0 01` → `M0`，`M0：` → `M0`）。
fn marker(text: &str) -> &str {
    let end = text
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(text.len());
    &text[..end]
}

/// 「票」欄標進行中的里程碑；最後一列的「下一步」給得出下一個 session 的指令。
fn check_session_rows(section: &Section<'_>, active: Option<&str>, violations: &mut Vec<String>) {
    let Some(table) = table(section) else {
        return;
    };
    let (Some(ticket), Some(next)) = (table.column("票"), table.column("下一步")) else {
        violations.push(format!(
            "{PROGRESS}:{}: session 紀錄的表頭要有「票」與「下一步」欄",
            table.header.number
        ));
        return;
    };
    for (number, cells) in &table.rows {
        let marker = marker(cells.get(ticket).copied().unwrap_or(""));
        check_marker(*number, marker, active, violations);
    }
    if let Some((number, cells)) = table.rows.last() {
        let next = cells.get(next).copied().unwrap_or("");
        if !gives_next_command(next) {
            violations.push(format!(
                "{PROGRESS}:{number}: 最後一列的「下一步」要含反引號包住、/ 開頭的指令（`/implement …`），或以「{WAITING}」開頭"
            ));
        }
    }
}

fn gives_next_command(next: &str) -> bool {
    next.starts_with(WAITING)
        || next
            .split('`')
            .skip(1)
            .step_by(2)
            .any(|code| code.starts_with('/'))
}

/// 每條一行：`- <YYYY-MM-DD> <里程碑> …`。
fn check_deviations(section: &Section<'_>, active: Option<&str>, violations: &mut Vec<String>) {
    for line in &section.body {
        let number = line.number;
        if line.text.trim().is_empty() {
            continue;
        }
        let Some(entry) = line.text.strip_prefix("- ") else {
            violations.push(format!(
                "{PROGRESS}:{number}: 偏差每條一行、以「- 」開頭，不分行也不用其他清單符號"
            ));
            continue;
        };
        let (date, rest) = entry.split_once(' ').unwrap_or((entry, ""));
        if !is_date(date) {
            violations.push(format!(
                "{PROGRESS}:{number}: 偏差每條以日期與里程碑開頭（`- 2026-10-30 M0 票 01：…`）"
            ));
            continue;
        }
        check_marker(number, marker(rest), active, violations);
    }
}

fn is_date(text: &str) -> bool {
    text.len() == 10
        && text.char_indices().all(|(index, c)| match index {
            4 | 7 => c == '-',
            _ => c.is_ascii_digit(),
        })
}

/// `.scratch/` 下的目錄：里程碑目錄（`m<數字>`）只給已開工的里程碑；有 `issues/` 的都要有完整的 spec，
/// 里程碑目錄的 `issues/` 還要有一張 `-close.md` 結尾的收尾票。
/// 其他名字的目錄是不屬於里程碑的工作（docs/agents/issue-tracker.md），不受里程碑表管。
fn check_scratch(
    root: &Path,
    milestones: &[Milestone<'_>],
    violations: &mut Vec<String>,
) -> Result<(), String> {
    let scratch = root.join(SCRATCH);
    if !scratch.is_dir() {
        return Ok(());
    }
    let read_error = |error| format!("讀不到 {SCRATCH}/：{error}");
    let mut names: Vec<String> = std::fs::read_dir(&scratch)
        .map_err(read_error)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_dir().then_some(entry)
        })
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    for name in &names {
        if is_milestone_dir(name) {
            check_milestone_started(name, milestones, violations);
        }
        let dir = scratch.join(name);
        if dir.join("issues").is_dir() {
            check_spec(&dir.join("spec.md"), name, violations);
            if is_milestone_dir(name) {
                check_close_ticket(&dir.join("issues"), name, violations)?;
            }
        }
    }
    Ok(())
}

/// 里程碑的 `issues/` 至少有一張 `-close.md` 結尾的收尾票；不要求它是編號最大的（收尾後補票是允許的）。
fn check_close_ticket(
    issues: &Path,
    name: &str,
    violations: &mut Vec<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(issues)
        .map_err(|error| format!("讀不到 {SCRATCH}/{name}/issues/：{error}"))?;
    let has_close = entries
        .filter_map(|entry| entry.ok())
        .any(|entry| entry.file_name().to_string_lossy().ends_with("-close.md"));
    if !has_close {
        violations.push(format!(
            "{SCRATCH}/{name}/issues/: 沒有收尾票：里程碑最後一張是 NN-{name}-close.md（CLAUDE.md〈Skills〉里程碑開工）"
        ));
    }
    Ok(())
}

/// `m` 加數字，例如 `m0`、`m12`。
fn is_milestone_dir(name: &str) -> bool {
    name.strip_prefix('m')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

fn check_milestone_started(name: &str, milestones: &[Milestone<'_>], violations: &mut Vec<String>) {
    match milestones
        .iter()
        .find(|milestone| milestone.id.to_lowercase() == name)
    {
        None => violations.push(format!(
            "{SCRATCH}/{name}/: 里程碑 {} 不在 {PROGRESS} 的里程碑表上：{SCRATCH}/ 只放已開工的里程碑",
            name.to_uppercase()
        )),
        Some(milestone) if milestone.not_started() => violations.push(format!(
            "{SCRATCH}/{name}/: 里程碑 {} 在 {PROGRESS} 的里程碑表是「未開始」：開工時才建目錄，並把備註改成進行中",
            milestone.id
        )),
        Some(_) => {}
    }
}

fn check_spec(path: &Path, name: &str, violations: &mut Vec<String>) {
    let spec = format!("{SCRATCH}/{name}/spec.md");
    let Ok(text) = std::fs::read_to_string(path) else {
        violations.push(format!("{spec}: 有 issues/ 就要有 spec.md"));
        return;
    };
    let sections = sections(&text);
    let has = |required: &str| {
        sections.iter().any(|section| {
            section.title == required
                || section
                    .title
                    .strip_prefix(required)
                    .is_some_and(|rest| rest.starts_with('（'))
        })
    };
    let all = SPEC_SECTIONS.map(|title| format!("〈{title}〉")).concat();
    for required in SPEC_SECTIONS {
        if !has(required) {
            violations.push(format!("{spec}: 缺〈{required}〉段：spec 要有{all}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // 變異測試用一份小型範本，不用 repo 現在的 progress：現在的內容會隨封存換掉，
    // 綁著它的測試會在封存那天一起壞。範本的格式照 docs/progress.md 與 .scratch/m0/spec.md。
    const PROGRESS_FIXTURE: &str = "\
# 進度

工作單位是票。

## 寫法與封存

- session 紀錄一列、偏差一條，整行各不超過 200 字元。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | plan §6 | 進行中。已拆 2 張票 |
| M1 | `.scratch/m1/issues/` | plan §7.2 | 未開始 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-01 | M0 拆票 | 寫 spec，拆成 2 張票 | 新 session 下 `/implement .scratch/m0/issues/01-a.md` |
| 2026-11-02 | M0 01 | 骨架與閘門 | 同上 |
| 2026-11-03 | M0 02 | 資料包 | 新 session 下 `/implement .scratch/m0/issues/03-c.md` |

## 偏差與決定

- 2026-11-01 M0 拆票：閘門一張做不完，拆成兩張。
- 2026-11-02 M0 票 01：`check` 加 `--fast`。
";

    const SPEC_FIXTURE: &str = "\
# M0 骨架

## 目標

repo 骨架與閘門就位。

## 範圍

### 做

- 閘門。

### 不做

- 連打。

## 共同規則

- 命名照名詞表。

## 驗收

- `cargo xtask check` 綠燈。

## 票序

| 票 | 被誰擋 | 交付 |
| --- | --- | --- |
| 01 | — | 骨架 |
";

    const SPEC: &str = ".scratch/m0/spec.md";
    const CLOSE: &str = ".scratch/m0/issues/02-m0-close.md";

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    /// 在暫存目錄寫出範本 repo：progress、`.scratch/m0/` 的 spec、一張票與收尾票。
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join(".scratch/m0/issues")).unwrap();
        fs::write(root.join(PROGRESS), PROGRESS_FIXTURE).unwrap();
        fs::write(root.join(SPEC), SPEC_FIXTURE).unwrap();
        fs::write(root.join(".scratch/m0/issues/01-a.md"), "# 01: a\n").unwrap();
        fs::write(root.join(CLOSE), "# 02: m0 close\n").unwrap();
        dir
    }

    fn progress(dir: &Path) -> String {
        fs::read_to_string(dir.join(PROGRESS)).unwrap()
    }

    /// 改寫副本裡的檔案；`edit` 沒改到任何東西就算測試寫錯。
    fn edit(dir: &Path, file: &str, edit: impl FnOnce(&str) -> String) {
        let path = dir.join(file);
        let before = fs::read_to_string(&path).unwrap();
        let after = edit(&before);
        assert_ne!(after, before, "變異沒有改到 {file}");
        fs::write(path, after).unwrap();
    }

    fn edit_progress(dir: &Path, change: impl FnOnce(&str) -> String) {
        edit(dir, PROGRESS, change);
    }

    /// `needle` 第一次出現在第幾行（從 1 起算）。
    fn line_of(text: &str, needle: &str) -> usize {
        let at = text.find(needle).expect("找不到要定位的字");
        text[..at].matches('\n').count() + 1
    }

    /// 違規要被抓到，且同一條說明提到 `expected` 的每個字串。
    fn assert_red(dir: &Path, expected: &[&str]) {
        let violations = run(dir).unwrap();
        assert!(
            violations
                .iter()
                .any(|v| expected.iter().all(|word| v.contains(word))),
            "應該有提到 {expected:?} 的違規，實際：{violations:#?}"
        );
    }

    fn assert_green(dir: &Path) {
        let violations = run(dir).unwrap();
        assert!(violations.is_empty(), "不應該有違規：{violations:#?}");
    }

    /// 整段〈Session 紀錄〉（標題到〈偏差與決定〉之前）。
    fn session_section(text: &str) -> String {
        let start = text.find("## Session 紀錄").unwrap();
        let end = start + text[start..].find("## 偏差與決定").unwrap();
        text[start..end].to_owned()
    }

    /// 在 session 紀錄表尾加一列（成為最後一列）。
    fn add_session_row(text: &str, row: &str) -> String {
        text.replacen("\n\n## 偏差與決定", &format!("\n{row}\n\n## 偏差與決定"), 1)
    }

    /// 在〈偏差與決定〉最後加一行（它是最後一段）。
    fn add_deviation(text: &str, entry: &str) -> String {
        format!("{text}{entry}\n")
    }

    /// 一列合規的 session 紀錄，用中文字補到整行剛好 `chars` 個字元。
    fn session_row(chars: usize) -> String {
        let row = |filler: &str| {
            format!("| 2026-11-03 | M0 04 | 結果{filler} | 新 session 下 `/implement x` |")
        };
        let filler = "字".repeat(chars - row("").chars().count());
        let row = row(&filler);
        assert_eq!(row.chars().count(), chars);
        row
    }

    /// 一條合規的偏差，用中文字補到整行剛好 `chars` 個字元。
    fn deviation(chars: usize) -> String {
        let prefix = "- 2026-11-03 M0 票 04：";
        let entry = format!("{prefix}{}", "字".repeat(chars - prefix.chars().count()));
        assert_eq!(entry.chars().count(), chars);
        entry
    }

    #[test]
    fn current_repo_passes() {
        assert_green(&repo_root());
    }

    #[test]
    fn fixture_passes() {
        assert_green(fixture().path());
    }

    // 〈##〉段

    #[test]
    fn duplicated_session_section_from_union_merge_is_red() {
        let dir = fixture();
        // 抄出的第二段插在原本〈偏差與決定〉的位置。
        let second = line_of(PROGRESS_FIXTURE, "## 偏差與決定");
        // union 合併兩邊都改了表尾時，會把整段再抄一次。
        edit_progress(dir.path(), |text| {
            let section = session_section(text);
            text.replacen("## 偏差與決定", &format!("{section}## 偏差與決定"), 1)
        });
        assert_red(
            dir.path(),
            &[&format!("{PROGRESS}:{second}:"), "〈Session 紀錄〉", "重複"],
        );
    }

    #[test]
    fn sections_out_of_order_are_red() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            let section = session_section(text);
            let without = text.replacen(&section, "", 1);
            format!("{without}\n{section}")
        });
        let line = line_of(&progress(dir.path()), "## Session 紀錄");
        assert_red(
            dir.path(),
            &[&format!("{PROGRESS}:{line}:"), "〈Session 紀錄〉", "順序"],
        );
    }

    #[test]
    fn missing_or_unknown_section_is_red() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen("## 寫法與封存", "## 寫法", 1)
        });
        assert_red(dir.path(), &["缺〈寫法與封存〉"]);
        assert_red(dir.path(), &["多出〈寫法〉"]);
    }

    #[test]
    fn headings_inside_code_blocks_are_not_sections() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen(
                "工作單位是票。\n",
                "工作單位是票。\n\n```md\n## Session 紀錄\n```\n\n~~~md\n```\n## 偏差與決定\n~~~\n",
                1,
            )
        });
        assert_green(dir.path());
    }

    // session 紀錄是一整張表

    #[test]
    fn session_table_broken_in_the_middle_is_red() {
        let dir = fixture();
        let row = "| 2026-11-02 | M0 01 |";
        let line = line_of(PROGRESS_FIXTURE, row);
        // 表格中間插一個空行：markdown 會把它當成兩張表，後半張沒有表頭。
        edit_progress(dir.path(), |text| {
            text.replacen(row, &format!("\n{row}"), 1)
        });
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "斷開"]);
    }

    #[test]
    fn text_between_session_rows_is_red() {
        let dir = fixture();
        let row = "| 2026-11-02 | M0 01 |";
        let line = line_of(PROGRESS_FIXTURE, row);
        edit_progress(dir.path(), |text| {
            text.replacen(row, &format!("中間的說明\n{row}"), 1)
        });
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "斷開"]);
    }

    #[test]
    fn text_after_the_session_table_is_red() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen("\n\n## 偏差與決定", "\n\n表後的說明\n\n## 偏差與決定", 1)
        });
        let line = line_of(&progress(dir.path()), "表後的說明");
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "表外"]);
    }

    #[test]
    fn session_section_without_a_table_is_red() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            let section = session_section(text);
            text.replacen(&section, "## Session 紀錄\n\n還沒有紀錄。\n\n", 1)
        });
        assert_red(dir.path(), &["〈Session 紀錄〉", "沒有表"]);
    }

    // 長度

    #[test]
    fn session_row_of_201_chars_is_red() {
        let dir = fixture();
        let row = session_row(201);
        edit_progress(dir.path(), |text| add_session_row(text, &row));
        let line = line_of(&progress(dir.path()), &row);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "201", "200"]);
    }

    #[test]
    fn deviation_of_201_chars_is_red() {
        let dir = fixture();
        let entry = deviation(201);
        edit_progress(dir.path(), |text| add_deviation(text, &entry));
        let line = line_of(&progress(dir.path()), &entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "201", "200"]);
    }

    #[test]
    fn rows_and_entries_of_exactly_200_chars_are_green() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            add_deviation(&add_session_row(text, &session_row(200)), &deviation(200))
        });
        assert_green(dir.path());
    }

    #[test]
    fn deviation_split_over_lines_or_other_bullets_is_red() {
        for (entry, culprit) in [
            ("- 2026-11-03 M0 票 04：前半。\n  後半。", "  後半。"),
            ("* 2026-11-03 M1 票 01：別的清單符號。", "* 2026-11-03"),
        ] {
            let dir = fixture();
            edit_progress(dir.path(), |text| add_deviation(text, entry));
            let line = line_of(&progress(dir.path()), culprit);
            assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "一行"]);
        }
    }

    // 里程碑標記

    #[test]
    fn session_row_marked_with_a_milestone_not_in_progress_is_red() {
        let dir = fixture();
        let row = "| 2026-11-03 | M1 01 | 黑箱基線 | 新 session 下 `/implement x` |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        let line = line_of(&progress(dir.path()), row);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "M1", "M0"]);
    }

    #[test]
    fn deviation_marked_with_a_milestone_not_in_progress_is_red() {
        let dir = fixture();
        let entry = "- 2026-11-03 M1 票 01：先做黑箱基線。";
        edit_progress(dir.path(), |text| add_deviation(text, entry));
        let line = line_of(&progress(dir.path()), entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "M1", "M0"]);
    }

    #[test]
    fn deviation_without_date_is_red() {
        let dir = fixture();
        let entry = "- M0 票 04：沒寫日期。";
        edit_progress(dir.path(), |text| add_deviation(text, entry));
        let line = line_of(&progress(dir.path()), entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "日期"]);
    }

    #[test]
    fn not_exactly_one_milestone_in_progress_is_red() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen("| 進行中。", "| 暫停。", 1)
        });
        assert_red(dir.path(), &["進行中", "0 個"]);

        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen("| 未開始 |", "| 進行中 |", 1)
        });
        assert_red(dir.path(), &["進行中", "2 個"]);
    }

    #[test]
    fn archiving_m0_while_opening_m1_is_green() {
        // 〈寫法與封存〉：封存 M0 的同一個 commit 把 M1 標成進行中；spec 與票留在原處。
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            let start = text.find("| 2026-11-01").unwrap();
            let end = text.find("\n\n## 偏差與決定").unwrap();
            let archived_rows = &text[start..end];
            let deviations = &text[text.find("- 2026-11-01").unwrap()..];
            text.replacen("進行中。已拆 2 張票", "[已封存](archive/m0.md)", 1)
                .replacen("| 未開始 |", "| 進行中。待拆票 |", 1)
                .replacen(
                    archived_rows,
                    "| 2026-11-04 | M1 開工 | 封存 M0 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |",
                    1,
                )
                .replacen(deviations, "", 1)
        });
        assert_green(dir.path());
    }

    // 下一步

    #[test]
    fn last_next_step_without_a_command_is_red() {
        for next in ["繼續做下一張票", "跑 `cargo xtask check`", "/implement x"] {
            let dir = fixture();
            let row = format!("| 2026-11-03 | M0 04 | 文件閘門 | {next} |");
            edit_progress(dir.path(), |text| add_session_row(text, &row));
            let line = line_of(&progress(dir.path()), &row);
            assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "下一步"]);
        }
    }

    #[test]
    fn last_next_step_waiting_for_the_user_is_green() {
        let dir = fixture();
        let row = "| 2026-11-03 | M0 04 | 文件閘門 | 等使用者：決定 M1 先做哪張票 |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        assert_green(dir.path());
    }

    #[test]
    fn escaped_pipe_does_not_lend_a_command_to_the_next_step() {
        // 不認 `\|` 的話，「下一步」欄會錯位成 `/foo`，看起來有指令。
        let dir = fixture();
        let row = r"| 2026-11-03 | M0 04 | x \| `/foo` | 繼續 |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        let line = line_of(&progress(dir.path()), row);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "下一步"]);
    }

    #[test]
    fn escaped_pipe_in_a_cell_does_not_shift_columns() {
        let dir = fixture();
        let row = r"| 2026-11-03 | M0 04 | 支援 `a \| b` 的寫法 | 新 session 下 `/implement x` |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        assert_green(dir.path());
    }

    #[test]
    fn earlier_rows_need_no_command() {
        let dir = fixture();
        let row = "| 2026-11-03 | M0 03 | 文件閘門 | 同上 |";
        let last = "| 2026-11-03 | M0 04 | 資料包 | 新 session 下 `/implement .scratch/m0/issues/05-e.md` |";
        edit_progress(dir.path(), |text| {
            add_session_row(&add_session_row(text, row), last)
        });
        assert_green(dir.path());
    }

    #[test]
    fn rewording_and_reformatting_rows_is_green() {
        let dir = fixture();
        edit_progress(dir.path(), |text| {
            text.replacen("骨架與閘門", "閘門與骨架", 1)
                .replacen("| --- | --- | --- | --- |", "|:---|:---|---|---:|", 1)
                .replacen("| M0 02 | 資料包 |", "|  M0 02  |  資料包  |", 1)
        });
        assert_green(dir.path());
    }

    // .scratch/

    #[test]
    fn issues_of_a_milestone_not_started_are_red() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join(".scratch/m1/issues")).unwrap();
        fs::write(dir.path().join(".scratch/m1/spec.md"), SPEC_FIXTURE).unwrap();
        assert_red(dir.path(), &[".scratch/m1/", "M1", "未開始"]);
    }

    #[test]
    fn milestone_directory_missing_from_the_table_is_red() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join(".scratch/m7")).unwrap();
        assert_red(dir.path(), &[".scratch/m7/", "M7"]);
    }

    #[test]
    fn spec_without_ticket_order_is_red() {
        let dir = fixture();
        edit(dir.path(), SPEC, |text| {
            text.replacen("## 票序", "## 票", 1)
        });
        assert_red(dir.path(), &[SPEC, "〈票序〉"]);
    }

    #[test]
    fn issues_without_spec_are_red() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join(".scratch/docs-cleanup/issues")).unwrap();
        assert_red(dir.path(), &[".scratch/docs-cleanup/spec.md"]);
    }

    #[test]
    fn rewriting_spec_body_is_green() {
        let dir = fixture();
        edit(dir.path(), SPEC, |text| {
            text.replacen("repo 骨架與閘門就位", "閘門與 repo 骨架先就位", 1)
                .replacen("### 做", "### 要做", 1)
                .replacen("## 範圍\n", "## 範圍（做 / 不做）\n\n範圍分兩塊。\n", 1)
        });
        assert_green(dir.path());
    }

    #[test]
    fn effort_directory_with_spec_is_green() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join(".scratch/docs-cleanup/issues")).unwrap();
        fs::write(
            dir.path().join(".scratch/docs-cleanup/spec.md"),
            SPEC_FIXTURE,
        )
        .unwrap();
        assert_green(dir.path());
    }

    #[test]
    fn milestone_without_close_ticket_is_red() {
        let dir = fixture();
        fs::remove_file(dir.path().join(CLOSE)).unwrap();
        assert_red(dir.path(), &[".scratch/m0/issues/", "收尾票"]);
    }

    #[test]
    fn rewriting_renumbering_or_following_the_close_ticket_is_green() {
        let dir = fixture();
        edit(dir.path(), CLOSE, |text| text.replacen("close", "收尾", 1));
        assert_green(dir.path());
        fs::rename(
            dir.path().join(CLOSE),
            dir.path().join(".scratch/m0/issues/07-m0-close.md"),
        )
        .unwrap();
        assert_green(dir.path());
        fs::write(dir.path().join(".scratch/m0/issues/08-b.md"), "# 08: b\n").unwrap();
        assert_green(dir.path());
    }

    #[test]
    fn effort_directory_without_close_ticket_is_green() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join(".scratch/docs-diet/issues")).unwrap();
        fs::write(dir.path().join(".scratch/docs-diet/spec.md"), SPEC_FIXTURE).unwrap();
        fs::write(
            dir.path().join(".scratch/docs-diet/issues/01-x.md"),
            "# 01: x\n",
        )
        .unwrap();
        assert_green(dir.path());
    }
}
