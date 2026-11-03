//! `docs` 步驟：`docs/progress.md` 與里程碑 spec 的規則（`docs/plan.md` §6）。

use std::path::Path;

const PROGRESS: &str = "docs/progress.md";

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

/// 里程碑 spec 與票所在的目錄。
const SCRATCH: &str = ".scratch";

/// 里程碑 spec 必備的 `##` 段。
const SPEC_SECTIONS: [&str; 5] = ["目標", "範圍", "共同規則", "驗收", "票序"];

/// progress 的 `##` 段，依序各一次。
const SECTIONS: [&str; 4] = [
    "寫法與封存",
    "里程碑與閱讀指引",
    "Session 紀錄",
    "偏差與決定",
];

/// 檢查 `root` 的 progress 與 `.scratch/`，回傳每一條違規的說明；空的就是通過。
pub fn run(root: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(root.join(PROGRESS))
        .map_err(|error| format!("讀不到 {PROGRESS}：{error}"))?;
    let sections = sections(&text);
    let mut violations = Vec::new();
    check_section_order(&sections, &mut violations);
    let milestones = section(&sections, "里程碑與閱讀指引")
        .map(|section| milestones(section, &mut violations))
        .unwrap_or_default();
    let active = active_milestone(&milestones, &mut violations);
    if let Some(session) = section(&sections, "Session 紀錄") {
        check_one_table(session, &mut violations);
        check_lengths(session, &mut violations);
        check_session_rows(session, active, &mut violations);
    }
    if let Some(deviations) = section(&sections, "偏差與決定") {
        check_lengths(deviations, &mut violations);
        check_deviations(deviations, active, &mut violations);
    }
    check_scratch(root, &milestones, &mut violations)?;
    Ok(violations)
}

/// session 紀錄每列、偏差每條的整行上限（Unicode 字元）。
const MAX_CHARS: usize = 200;

/// 一行內容與它的行號（從 1 起算）。
type Line<'a> = (usize, &'a str);

/// 一個 `##` 段：標題、標題所在行、到下一個 `##` 之前的內容。
struct Section<'a> {
    title: &'a str,
    line: usize,
    body: Vec<Line<'a>>,
}

/// 切出 `##` 段；第一個 `##` 之前的內容不屬於任何段。程式碼區塊內的 `##` 不算標題。
fn sections(text: &str) -> Vec<Section<'_>> {
    let mut sections: Vec<Section<'_>> = Vec::new();
    let mut in_fence = false;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        match line.strip_prefix("## ") {
            Some(title) if !in_fence => sections.push(Section {
                title: title.trim(),
                line: number,
                body: Vec::new(),
            }),
            _ => {
                if let Some(section) = sections.last_mut() {
                    section.body.push((number, line));
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

fn is_table_row(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

/// 段內只有一張表：第一列到最後一列之間每行都是表格列，表外沒有其他文字。
fn check_one_table(section: &Section<'_>, violations: &mut Vec<String>) {
    let title = section.title;
    let rows = || section.body.iter().filter(|(_, line)| is_table_row(line));
    let (Some(&(first, _)), Some(&(last, _))) = (rows().next(), rows().next_back()) else {
        violations.push(format!(
            "{PROGRESS}:{}: 〈{title}〉要是一整張表，現在沒有表",
            section.line
        ));
        return;
    };
    for &(number, line) in &section.body {
        if is_table_row(line) {
            continue;
        }
        if (first..last).contains(&number) {
            violations.push(format!(
                "{PROGRESS}:{number}: 〈{title}〉要是一整張表，表格在這一行斷開"
            ));
        } else if !line.trim().is_empty() {
            violations.push(format!(
                "{PROGRESS}:{number}: 〈{title}〉要是一整張表，表外不放文字"
            ));
        }
    }
}

/// 段內每一行不超過 [`MAX_CHARS`] 個 Unicode 字元。
fn check_lengths(section: &Section<'_>, violations: &mut Vec<String>) {
    for &(number, line) in &section.body {
        let chars = line.chars().count();
        if chars > MAX_CHARS {
            violations.push(format!(
                "{PROGRESS}:{number}: 整行 {chars} 字元，超過 {MAX_CHARS}：細節移到票的 ## Comments，這裡只留一句指向它"
            ));
        }
    }
}

/// 段內第一張表的表頭與資料列（略過分隔列）；資料列附行號。
struct Table<'a> {
    header: Line<'a>,
    rows: Vec<(usize, Vec<&'a str>)>,
}

impl Table<'_> {
    fn column(&self, name: &str) -> Option<usize> {
        cells(self.header.1).iter().position(|cell| *cell == name)
    }
}

fn table<'a>(section: &Section<'a>) -> Option<Table<'a>> {
    let mut lines = section
        .body
        .iter()
        .skip_while(|(_, line)| !is_table_row(line))
        .take_while(|(_, line)| is_table_row(line));
    let header = *lines.next()?;
    let rows = lines
        .map(|&(number, line)| (number, cells(line)))
        .filter(|(_, cells)| {
            !cells
                .iter()
                .all(|cell| !cell.is_empty() && cell.chars().all(|c| matches!(c, '-' | ':')))
        })
        .collect();
    Some(Table { header, rows })
}

/// 表格列的各欄，去掉前後空白。
fn cells(line: &str) -> Vec<&str> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    line.split('|').map(str::trim).collect()
}

/// 里程碑表的一列。
struct Milestone<'a> {
    id: &'a str,
    note: &'a str,
}

/// 讀〈里程碑與閱讀指引〉的表：依表頭找「里程碑」與「備註」欄。
fn milestones<'a>(section: &Section<'a>, violations: &mut Vec<String>) -> Vec<Milestone<'a>> {
    let line = section.line;
    let Some(table) = table(section) else {
        violations.push(format!(
            "{PROGRESS}:{line}: 〈{}〉缺里程碑表",
            section.title
        ));
        return Vec::new();
    };
    let (Some(id), Some(note)) = (table.column("里程碑"), table.column("備註")) else {
        violations.push(format!(
            "{PROGRESS}:{}: 里程碑表的表頭要有「里程碑」與「備註」欄",
            table.header.0
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
fn active_milestone<'a>(
    milestones: &[Milestone<'a>],
    violations: &mut Vec<String>,
) -> Option<&'a str> {
    let active: Vec<&str> = milestones
        .iter()
        .filter(|milestone| milestone.note.starts_with("進行中"))
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
            table.header.0
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

/// 「下一步」可以等使用者決定的開頭。
const WAITING: &str = "等使用者：";

fn gives_next_command(next: &str) -> bool {
    next.starts_with(WAITING)
        || next
            .split('`')
            .skip(1)
            .step_by(2)
            .any(|code| code.starts_with('/'))
}

/// 每條是 `- <YYYY-MM-DD> <里程碑> …`；縮排的續行不算新的一條。
fn check_deviations(section: &Section<'_>, active: Option<&str>, violations: &mut Vec<String>) {
    for &(number, line) in &section.body {
        let Some(entry) = line.strip_prefix("- ") else {
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

/// `.scratch/` 下有 `issues/` 的目錄都有完整的 spec；里程碑目錄（`m<數字>`）只給已開工的里程碑。
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
        let dir = scratch.join(name);
        if is_milestone_dir(name) {
            match milestones
                .iter()
                .find(|milestone| milestone.id.to_lowercase() == *name)
            {
                None => violations.push(format!(
                    "{SCRATCH}/{name}/: 里程碑 {} 不在 {PROGRESS} 的里程碑表上：{SCRATCH}/ 只放已開工的里程碑",
                    name.to_uppercase()
                )),
                Some(milestone) if milestone.note.starts_with("未開始") => {
                    violations.push(format!(
                        "{SCRATCH}/{name}/: 里程碑 {} 在 {PROGRESS} 的里程碑表是「未開始」：開工時才建目錄，並把備註改成進行中",
                        milestone.id
                    ));
                }
                Some(_) => {}
            }
        }
        if !dir.join("issues").is_dir() {
            continue;
        }
        let spec = format!("{SCRATCH}/{name}/spec.md");
        let Ok(text) = std::fs::read_to_string(dir.join("spec.md")) else {
            violations.push(format!("{spec}: 有 issues/ 就要有 spec.md"));
            continue;
        };
        let titles: Vec<&str> = sections(&text)
            .iter()
            .map(|section| section.title)
            .collect();
        for title in SPEC_SECTIONS {
            if !titles.contains(&title) {
                let all = SPEC_SECTIONS.map(|title| format!("〈{title}〉")).concat();
                violations.push(format!("{spec}: 缺〈{title}〉段：spec 要有{all}"));
            }
        }
    }
    Ok(())
}

/// `m` 加數字，例如 `m0`、`m12`。
fn is_milestone_dir(name: &str) -> bool {
    name.strip_prefix('m')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

fn check_section_order(sections: &[Section<'_>], violations: &mut Vec<String>) {
    let order = SECTIONS.map(|title| format!("〈{title}〉")).concat();
    let mut seen: Vec<usize> = Vec::new();
    for section in sections {
        let title = section.title;
        let line = section.line;
        let Some(index) = SECTIONS.iter().position(|known| *known == title) else {
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
    for (index, title) in SECTIONS.iter().enumerate() {
        if !seen.contains(&index) {
            violations.push(format!("{PROGRESS}: 缺〈{title}〉段：## 段依序是{order}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    /// progress 與 `.scratch/` 複製到暫存目錄，變異只動副本。
    fn copy_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = repo_root();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::copy(root.join(PROGRESS), dir.path().join(PROGRESS)).unwrap();
        copy_dir(&root.join(".scratch"), &dir.path().join(".scratch"));
        dir
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    fn progress(dir: &Path) -> String {
        fs::read_to_string(dir.join(PROGRESS)).unwrap()
    }

    /// 改寫副本的 progress；`edit` 沒改到任何東西就算測試寫錯。
    fn edit_progress(dir: &Path, edit: impl FnOnce(&str) -> String) {
        let before = progress(dir);
        let after = edit(&before);
        assert_ne!(after, before, "變異沒有改到 progress");
        fs::write(dir.join(PROGRESS), after).unwrap();
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

    /// 整段〈Session 紀錄〉（標題到下一個 `##` 之前）。
    fn session_section(text: &str) -> String {
        let start = text.find("## Session 紀錄").unwrap();
        let end = start + text[start..].find("## 偏差與決定").unwrap();
        text[start..end].to_owned()
    }

    /// 在 session 紀錄表尾加一列（成為最後一列）。
    fn add_session_row(text: &str, row: &str) -> String {
        text.replacen("\n\n## 偏差與決定", &format!("\n{row}\n\n## 偏差與決定"), 1)
    }

    /// 在〈偏差與決定〉最後加一條（它是最後一段）。
    fn add_deviation(text: &str, entry: &str) -> String {
        assert!(text.ends_with('\n'));
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
    fn duplicated_session_section_from_union_merge_is_red() {
        let dir = copy_repo();
        // 抄出的第二段插在原本〈偏差與決定〉的位置。
        let second = line_of(&progress(dir.path()), "## 偏差與決定");
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
        let dir = copy_repo();
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
    fn missing_section_is_red() {
        let dir = copy_repo();
        edit_progress(dir.path(), |text| {
            text.replacen("## 寫法與封存", "## 寫法", 1)
        });
        assert_red(dir.path(), &["缺〈寫法與封存〉"]);
        assert_red(dir.path(), &["多出〈寫法〉"]);
    }

    #[test]
    fn session_table_broken_in_the_middle_is_red() {
        let dir = copy_repo();
        let original = progress(dir.path());
        let row = "| 2026-10-31 | M0 plan | 加入畫布式佈局";
        let line = line_of(&original, row);
        // 表格中間插一個空行：markdown 會把它當成兩張表，後半張沒有表頭。
        edit_progress(dir.path(), |text| {
            text.replacen(row, &format!("\n{row}"), 1)
        });
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "一整張表"]);
    }

    #[test]
    fn text_between_session_rows_is_red() {
        let dir = copy_repo();
        let row = "| 2026-10-31 | M0 plan | 加入畫布式佈局";
        let line = line_of(&progress(dir.path()), row);
        edit_progress(dir.path(), |text| {
            text.replacen(row, &format!("中間的說明\n{row}"), 1)
        });
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "一整張表"]);
    }

    #[test]
    fn session_row_of_201_chars_is_red() {
        let dir = copy_repo();
        let row = session_row(201);
        edit_progress(dir.path(), |text| add_session_row(text, &row));
        let line = line_of(&progress(dir.path()), &row);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "201", "200"]);
    }

    #[test]
    fn deviation_of_201_chars_is_red() {
        let dir = copy_repo();
        let entry = deviation(201);
        edit_progress(dir.path(), |text| add_deviation(text, &entry));
        let line = line_of(&progress(dir.path()), &entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "201", "200"]);
    }

    #[test]
    fn rows_and_entries_of_exactly_200_chars_are_green() {
        let dir = copy_repo();
        let entry = deviation(200);
        edit_progress(dir.path(), |text| {
            add_deviation(&add_session_row(text, &session_row(200)), &entry)
        });
        assert_green(dir.path());
    }

    #[test]
    fn session_row_marked_with_a_milestone_not_in_progress_is_red() {
        let dir = copy_repo();
        let row = "| 2026-11-03 | M1 01 | 黑箱基線 | 新 session 下 `/implement x` |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        let line = line_of(&progress(dir.path()), row);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "M1", "M0"]);
    }

    #[test]
    fn deviation_marked_with_a_milestone_not_in_progress_is_red() {
        let dir = copy_repo();
        let entry = "- 2026-11-03 M1 票 01：先做黑箱基線。";
        edit_progress(dir.path(), |text| add_deviation(text, entry));
        let line = line_of(&progress(dir.path()), entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "M1", "M0"]);
    }

    #[test]
    fn deviation_without_date_is_red() {
        let dir = copy_repo();
        let entry = "- M0 票 04：沒寫日期。";
        edit_progress(dir.path(), |text| add_deviation(text, entry));
        let line = line_of(&progress(dir.path()), entry);
        assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "日期"]);
    }

    #[test]
    fn no_milestone_in_progress_is_red() {
        let dir = copy_repo();
        edit_progress(dir.path(), |text| {
            text.replacen("| 進行中。", "| 暫停。", 1)
        });
        assert_red(dir.path(), &["進行中", "0 個"]);
    }

    #[test]
    fn last_next_step_without_a_command_is_red() {
        for next in ["繼續做下一張票", "跑 `cargo xtask check`", "/implement x"] {
            let dir = copy_repo();
            let row = format!("| 2026-11-03 | M0 04 | 文件閘門 | {next} |");
            edit_progress(dir.path(), |text| add_session_row(text, &row));
            let line = line_of(&progress(dir.path()), &row);
            assert_red(dir.path(), &[&format!("{PROGRESS}:{line}:"), "下一步"]);
        }
    }

    #[test]
    fn last_next_step_waiting_for_the_user_is_green() {
        let dir = copy_repo();
        let row = "| 2026-11-03 | M0 04 | 文件閘門 | 等使用者：決定 M1 先做哪張票 |";
        edit_progress(dir.path(), |text| add_session_row(text, row));
        assert_green(dir.path());
    }

    #[test]
    fn earlier_rows_need_no_command() {
        let dir = copy_repo();
        let row = "| 2026-11-03 | M0 04 | 文件閘門 | 同上 |";
        let last = "| 2026-11-03 | M0 05 | 資料包 | 新 session 下 `/implement .scratch/m0/issues/06-unihan-cross-check.md` |";
        edit_progress(dir.path(), |text| {
            add_session_row(&add_session_row(text, row), last)
        });
        assert_green(dir.path());
    }

    #[test]
    fn rewording_a_row_is_green() {
        let dir = copy_repo();
        edit_progress(dir.path(), |text| {
            text.replacen(
                "參照 Berth 建立文件骨架",
                "參考 Berth 的做法建立文件骨架",
                1,
            )
            .replacen("|", "| ", 1)
        });
        assert_green(dir.path());
    }

    const SPEC: &str = ".scratch/m0/spec.md";

    fn edit_spec(dir: &Path, edit: impl FnOnce(&str) -> String) {
        let path = dir.join(SPEC);
        let before = fs::read_to_string(&path).unwrap();
        let after = edit(&before);
        assert_ne!(after, before, "變異沒有改到 spec");
        fs::write(path, after).unwrap();
    }

    #[test]
    fn issues_of_a_milestone_not_started_are_red() {
        let dir = copy_repo();
        fs::create_dir_all(dir.path().join(".scratch/m1/issues")).unwrap();
        fs::copy(
            dir.path().join(SPEC),
            dir.path().join(".scratch/m1/spec.md"),
        )
        .unwrap();
        assert_red(dir.path(), &[".scratch/m1/", "M1", "未開始"]);
    }

    #[test]
    fn milestone_directory_missing_from_the_table_is_red() {
        let dir = copy_repo();
        fs::create_dir_all(dir.path().join(".scratch/m7")).unwrap();
        assert_red(dir.path(), &[".scratch/m7/", "M7"]);
    }

    #[test]
    fn spec_without_ticket_order_is_red() {
        let dir = copy_repo();
        edit_spec(dir.path(), |text| text.replacen("## 票序", "## 票", 1));
        assert_red(dir.path(), &[SPEC, "〈票序〉"]);
    }

    #[test]
    fn issues_without_spec_are_red() {
        let dir = copy_repo();
        fs::create_dir_all(dir.path().join(".scratch/docs-cleanup/issues")).unwrap();
        assert_red(dir.path(), &[".scratch/docs-cleanup/spec.md"]);
    }

    #[test]
    fn rewriting_spec_body_is_green() {
        let dir = copy_repo();
        edit_spec(dir.path(), |text| {
            text.replacen("repo 骨架與閘門就位", "閘門與 repo 骨架先就位", 1)
                .replacen("### 做", "### 要做", 1)
                .replacen("## 範圍\n", "## 範圍\n\n範圍分兩塊。\n", 1)
        });
        assert_green(dir.path());
    }

    #[test]
    fn effort_directory_with_spec_is_green() {
        let dir = copy_repo();
        fs::create_dir_all(dir.path().join(".scratch/docs-cleanup/issues")).unwrap();
        fs::copy(
            dir.path().join(SPEC),
            dir.path().join(".scratch/docs-cleanup/spec.md"),
        )
        .unwrap();
        assert_green(dir.path());
    }
}
