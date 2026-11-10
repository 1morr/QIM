//! 殼的觸控鍵盤狀態：選到的佈局版本、目前頁、觸控 Shift，與點按之後要交給核心的輸入。
//! 各平台殼共用，綁定只轉型別（plan §1.2、§2.6，M2 票 03）。

use crate::model::{Action, Key, KeyState, Layout, LayoutVariant, Page, Shift, When};
use crate::validate::Warning;

/// 一份佈局在殼上的狀態。切頁與 Shift 在 [`Keyboard::tap`]、[`Keyboard::long_press`] 裡做完，
/// 殼只把回傳的 [`Output`] 交給核心、照 [`Keyboard::page`] 畫。
#[derive(Debug, Clone)]
pub struct Keyboard {
    layout: Layout,
    /// 選到的佈局版本與目前頁在陣列的位置。
    variant: usize,
    page: usize,
    shift: Shift,
}

/// 觸控鍵盤點按後送出、殼交給核心的東西。只改鍵盤狀態的動作（切頁、⇧、換鍵盤）已在 [`Keyboard`] 做完，
/// 剩下 `Modifier`：核心組字中吃下、沒有組字時交還宿主但不結束數字（brief §4.7）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    /// 字母鍵，同實體鍵盤按那個字母。
    Letter(char),
    /// ASCII 標點鍵，同實體鍵盤按那個鍵：全形、半形與組字中的處理由核心決定。
    Punct(char),
    /// 直接輸入的文字，不經組字（核心的 `Session::text`）。
    Text(String),
    Space,
    Backspace,
    Enter,
    /// 觸控的中英鍵，同實體鍵盤單按右 Shift（先上屏組字，M1 票 23）。
    ToggleAscii,
    /// 只改了鍵盤狀態的鍵。
    Modifier,
}

impl Output {
    /// 種類的名字（同核心 `Key::parse` 讀的種類）；殼在核心沒吃下時照它做宿主的預設動作。
    pub fn kind(&self) -> &'static str {
        match self {
            Output::Letter(_) => "letter",
            Output::Punct(_) => "punct",
            Output::Text(_) => "text",
            Output::Space => "space",
            Output::Backspace => "backspace",
            Output::Enter => "enter",
            Output::ToggleAscii => "toggleAscii",
            Output::Modifier => "modifier",
        }
    }

    /// 字母與標點鍵的字元；核心沒吃下時宿主照打它。
    pub fn character(&self) -> Option<char> {
        match self {
            Output::Letter(c) | Output::Punct(c) => Some(*c),
            _ => None,
        }
    }
}

/// 點按了目前頁沒有的鍵或長按清單沒有的選項。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardError {
    NoKey { index: usize },
    NoLongPressChoice { key: String, choice: usize },
}

impl std::fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardError::NoKey { index } => write!(f, "佈局目前頁沒有第 {index} 個鍵"),
            KeyboardError::NoLongPressChoice { key, choice } => {
                write!(f, "鍵 {key} 的長按清單沒有第 {choice} 個")
            }
        }
    }
}

impl std::error::Error for KeyboardError {}

impl Keyboard {
    /// 依裝置與方向選佈局版本（[`Layout::select`]），顯示它一開始的頁、Shift 關著；
    /// 沒有相符的版本時用第一個版本並回傳警告。`layout` 要是驗證過的（[`Layout::from_json`]）。
    pub fn new(layout: Layout, when: When) -> (Keyboard, Option<Warning>) {
        let (variant, warning) = layout.select(when);
        let mut keyboard = Keyboard {
            layout,
            variant,
            page: 0,
            shift: Shift::Off,
        };
        keyboard.show_page_or_initial(None);
        (keyboard, warning)
    }

    /// 依裝置與方向重選佈局版本（轉向時）：新版本有同 id 的頁就留在那頁，否則回到一開始的頁。
    pub fn select(&mut self, when: When) -> Option<Warning> {
        let current = self.page().id.clone();
        let (variant, warning) = self.layout.select(when);
        self.variant = variant;
        self.show_page_or_initial(Some(&current));
        warning
    }

    /// 換成同一個鍵盤的另一份佈局（數字列開關）時接手 `old` 的 Shift 與目前頁：有同 id 的頁就留在
    /// 那頁，否則回到一開始的頁（同轉向，M1 票 27b）。
    pub fn keep_state_of(&mut self, old: &Keyboard) {
        self.shift = old.shift;
        self.show_page_or_initial(Some(&old.page().id));
    }

    pub fn variant(&self) -> &LayoutVariant {
        &self.layout.variants[self.variant]
    }

    /// 目前頁；殼照它畫、用 [`Page::hit`] 判定點到第幾個鍵。
    pub fn page(&self) -> &Page {
        &self.variant().pages[self.page]
    }

    pub fn shift(&self) -> Shift {
        self.shift
    }

    /// 選標籤與動作用的狀態；`composing` 由核心的 `Session::composing` 來。
    pub fn key_state(&self, composing: bool) -> KeyState {
        KeyState {
            composing,
            shift: self.shift,
        }
    }

    /// 點一下目前頁第 `index` 個鍵：做它的 [`Key::tap_action`]（組字中照 `composing`，Shift 開著時
    /// 字母鍵輸入大寫字母）。`quick_repeat` 是同一個鍵剛點過（殼量時間），快速點兩下 Shift 鎖定。
    pub fn tap(
        &mut self,
        index: usize,
        composing: bool,
        quick_repeat: bool,
    ) -> Result<Output, KeyboardError> {
        let action = self.key(index)?.tap_action(self.key_state(composing));
        self.shift = match action {
            Action::Shift => self.shift.after_shift_tap(quick_repeat),
            _ => self.shift.after_other_key(),
        };
        Ok(self.perform(action))
    }

    /// 長按目前頁第 `index` 個鍵、選了長按清單第 `choice` 個（0 是預設）。
    pub fn long_press(&mut self, index: usize, choice: usize) -> Result<Output, KeyboardError> {
        let key = self.key(index)?;
        let action = key.long_press.get(choice).cloned().ok_or_else(|| {
            KeyboardError::NoLongPressChoice {
                key: key.id.clone(),
                choice,
            }
        })?;
        self.shift = self.shift.after_other_key();
        Ok(self.perform(action))
    }

    /// 做動作裡改鍵盤狀態的部分（切頁），回傳要交給核心的輸入。
    fn perform(&mut self, action: Action) -> Output {
        match action {
            Action::Letter(c) => Output::Letter(c),
            Action::Punct(c) => Output::Punct(c),
            Action::Text(text) => Output::Text(text),
            Action::Space => Output::Space,
            Action::Backspace => Output::Backspace,
            Action::Enter => Output::Enter,
            Action::ToggleAscii => Output::ToggleAscii,
            Action::SwitchPage(id) => {
                // `validate` 保證 `switchPage` 指到的頁存在。
                if let Some(page) = self.variant().page_index(&id) {
                    self.page = page;
                }
                Output::Modifier
            }
            // ⇧ 的狀態在 `tap` 改；換鍵盤由殼接系統的切換。
            Action::Shift | Action::NextKeyboard => Output::Modifier,
        }
    }

    fn key(&self, index: usize) -> Result<&Key, KeyboardError> {
        self.page()
            .keys
            .get(index)
            .ok_or(KeyboardError::NoKey { index })
    }

    /// 目前版本有 `id` 這頁就切過去，否則回到一開始的頁。
    fn show_page_or_initial(&mut self, id: Option<&str>) {
        self.page = self
            .variant()
            .page_index_or_initial(id)
            .expect("validate 保證 initialPage 存在、每個版本至少一頁");
    }
}
