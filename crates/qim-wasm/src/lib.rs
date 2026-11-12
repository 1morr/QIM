//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core`、`qim-layout` 與 `qim-neural`，不放邏輯（plan §1.3）。

use qim_core::{
    CandidateKind, CangjieVersion, ContextState, Date, Engine, Imported, IndexEntry, Inspection,
    Key, Manifest, ModelPack, Options, Pack, PackIndex, PrecedingChar, RadicalHintPack,
    RadicalHints, Scheme, SegmentSource, Session, ShiftKey, Shortcuts, Snapshot, UserModel,
    Weights, WordListReport,
};
use qim_layout::{Action, Keyboard, Output, Target, Touch, When};
use wasm_bindgen::prelude::*;

/// 核心與 `qim-layout` 的錯誤轉成 JS 的例外，訊息照原樣。
fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

/// 匯入學習資料略過的列，每條是「第 N 行：原因」。
fn warnings(imported: &Imported) -> Vec<String> {
    imported.warnings.iter().map(ToString::to_string).collect()
}

/// 一份資料包：`manifest.json` 的內容與資料包檔的位元組（plan §4.6），網頁 fetch 或從檔案、
/// IndexedDB 讀來。建立時只讀 manifest；核對大小、sha256 與解開在 [`WebPack::verify`] 或交給
/// `Session` 時（核心的 [`Pack::open`]）。JS 端叫 `Pack`。
#[wasm_bindgen(js_name = Pack)]
pub struct WebPack {
    manifest: Manifest,
    bytes: Vec<u8>,
}

#[wasm_bindgen(js_class = Pack)]
impl WebPack {
    #[wasm_bindgen(constructor)]
    pub fn new(manifest: &str, bytes: Vec<u8>) -> Result<WebPack, JsError> {
        Ok(Self {
            manifest: Manifest::from_json(manifest).map_err(js_error)?,
            bytes,
        })
    }

    /// 照 manifest 核對並解開一次，不留結果：匯入前檢查用。
    pub fn verify(&self) -> Result<(), JsError> {
        self.manifest.verify(&self.bytes).map_err(js_error)
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.manifest.id.clone()
    }

    /// manifest 的 `kind`（`lexicon`、`language-model`…）。
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.manifest.kind.as_str().into()
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.manifest.version.clone()
    }

    /// 資料包檔的位元組數（manifest 寫的）。
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> f64 {
        self.manifest.size as f64
    }

    #[wasm_bindgen(getter)]
    pub fn license(&self) -> String {
        self.manifest.license.clone()
    }

    /// 資料包檔的檔名 `<id>.pack`。
    #[wasm_bindgen(getter, js_name = fileName)]
    pub fn file_name(&self) -> String {
        self.manifest.file_name()
    }
}

impl WebPack {
    fn open<P: Pack>(&self) -> Result<P, JsError> {
        P::open(&self.manifest, &self.bytes).map_err(js_error)
    }
}

/// 更新索引裡要下載的一項（plan §4.7）：新版的 manifest 與資料包檔的位置。
#[wasm_bindgen]
pub struct PackUpdate(IndexEntry);

#[wasm_bindgen]
impl PackUpdate {
    /// 新版的 `manifest.json`（不含 `url`），下載後和位元組一起交給 `Pack`。
    #[wasm_bindgen(getter)]
    pub fn manifest(&self) -> String {
        self.0.manifest.to_json()
    }

    /// 資料包檔的位置，相對於索引的網址。
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.0.url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.0.manifest.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.0.manifest.version.clone()
    }
}

/// `manifest` 的版本比 `than` 的大（兩個都是 `manifest.json` 的內容；核心的 [`Manifest::is_newer_than`]）。
#[wasm_bindgen(js_name = manifestIsNewer)]
pub fn manifest_is_newer(manifest: &str, than: &str) -> Result<bool, JsError> {
    let manifest = Manifest::from_json(manifest).map_err(js_error)?;
    let than = Manifest::from_json(than).map_err(js_error)?;
    Ok(manifest.is_newer_than(&than))
}

/// 更新索引 `index`（`index.json` 的內容）裡，`installed`（裝著的資料包的 `manifest.json`）有新版的
/// 那些（核心的 [`PackIndex::updates`]）。
#[wasm_bindgen(js_name = packUpdates)]
pub fn pack_updates(index: &str, installed: Vec<String>) -> Result<Vec<PackUpdate>, JsError> {
    let index = PackIndex::from_json(index).map_err(js_error)?;
    let installed = installed
        .iter()
        .map(|json| Manifest::from_json(json))
        .collect::<Result<Vec<_>, _>>()
        .map_err(js_error)?;
    Ok(index
        .updates(&installed)
        .into_iter()
        .map(|entry| PackUpdate(entry.clone()))
        .collect())
}

/// 由一組資料包建 `Engine`（核心的 [`Engine::open`]）。
fn engine(packs: &[WebPack]) -> Result<Engine, JsError> {
    let files: Vec<_> = packs
        .iter()
        .map(|p| (&p.manifest, p.bytes.as_slice()))
        .collect();
    Engine::open(&files).map_err(js_error)
}

/// 網頁的一個輸入框：速成、詞庫、語言模型、詞級 trigram、簡繁、英文與 emoji 資料包、一個個人模型、一個 `Session`。
/// JS 端叫 `Session`。
#[wasm_bindgen(js_name = Session)]
pub struct WebSession {
    engine: Engine,
    user: UserModel,
    session: Session,
}

#[wasm_bindgen(js_class = Session)]
impl WebSession {
    /// 由一組資料包建立：方案資料包，加上方案要的詞庫、語言模型、詞級 trigram，與可有可無的簡繁（沒有時簡體
    /// 模式照原樣輸出）、英文（沒有時沒有英文候選）、emoji（沒有時沒有 emoji 候選）資料包，順序
    /// 不拘。資料包交給 `Session` 後在 JS 端就不能再用。
    #[wasm_bindgen(constructor)]
    pub fn new(packs: Vec<WebPack>) -> Result<WebSession, JsError> {
        Ok(Self {
            engine: engine(&packs)?,
            user: UserModel::default(),
            session: Session::default(),
        })
    }

    /// 換資料包（設定頁的切換、匯入、刪除，plan §4.6）：參數同建構子。學習資料、選項與組字留著，
    /// 組字照新的資料包重列候選；任一份核對不過時整組不換、原本的照用。
    #[wasm_bindgen(js_name = setPacks)]
    pub fn set_packs(&mut self, packs: Vec<WebPack>) -> Result<WebSnapshot, JsError> {
        self.engine = engine(&packs)?;
        Ok(self.session.refresh(&self.engine, &self.user).into())
    }

    /// 無痕模式：開著時不學習（plan §2.4）、不用前文（brief §8）。
    #[wasm_bindgen(js_name = setIncognito)]
    pub fn set_incognito(&mut self, on: bool) {
        self.set_options(|options| options.incognito = on);
    }

    /// 學習（brief §4.5、§7）：關掉時不寫任何學習資料，已學的照用、前文照用。預設開。
    #[wasm_bindgen(js_name = setLearning)]
    pub fn set_learning(&mut self, on: bool) {
        self.set_options(|options| options.learning = on);
    }

    /// 無痕模式開著：網頁這時不給前文（brief §8）。
    #[wasm_bindgen(getter)]
    pub fn incognito(&self) -> bool {
        self.session.options().incognito
    }

    /// 輸入框游標前的文字（`Session::set_context`）：網頁在聚焦、移動游標、輸入框改了之後給。
    #[wasm_bindgen(js_name = setContext)]
    pub fn set_context(&mut self, before: &str) {
        self.session.set_context(before);
    }

    /// 忘掉前文（`Session::clear_context`）：輸入框失焦時；正在打的 email 這時算打完、記住。
    #[wasm_bindgen(js_name = clearContext)]
    pub fn clear_context(&mut self) {
        self.session.clear_context(&mut self.user);
    }

    /// 手動加一個使用者詞；`trigger` 是空字串時不設觸發詞（brief §4.5）。
    #[wasm_bindgen(js_name = addWord)]
    pub fn add_word(&mut self, text: &str, trigger: &str) -> Result<(), JsError> {
        let trigger = (!trigger.is_empty()).then_some(trigger);
        self.user
            .add_word(&self.engine, text, trigger)
            .map_err(js_error)
    }

    /// 存一條常用語；`trigger` 是空字串時沒有觸發詞，只能從清單選（plan §2.4）。同樣的文字再存
    /// 一次換上新的觸發詞。
    #[wasm_bindgen(js_name = addPhrase)]
    pub fn add_phrase(&mut self, text: &str, trigger: &str) -> Result<(), JsError> {
        let trigger = (!trigger.is_empty()).then_some(trigger);
        self.user
            .add_phrase(&self.engine, text, trigger)
            .map_err(js_error)
    }

    /// 刪掉文字是 `text` 的常用語；沒有這條時回傳 `false`。
    #[wasm_bindgen(js_name = removePhrase)]
    pub fn remove_phrase(&mut self, text: &str) -> bool {
        self.user.remove_phrase(&self.engine, text)
    }

    /// 常用語的清單，依文字排。
    pub fn phrases(&self) -> Vec<WebPhrase> {
        self.user
            .phrases(&self.engine)
            .into_iter()
            .map(|p| WebPhrase {
                text: p.text,
                trigger: p.trigger.unwrap_or_default(),
            })
            .collect()
    }

    /// 英文使用者詞（M4 票 14，打過的英文串，沒到出現門檻的也列），依小寫排；設定區的清單用。
    #[wasm_bindgen(js_name = englishUserWords)]
    pub fn english_user_words(&self) -> Vec<String> {
        self.user.english_user_words()
    }

    /// 忘掉英文使用者詞 `text`（不分大小寫）；沒學過時回傳 `false`。
    #[wasm_bindgen(js_name = forgetEnglishUserWord)]
    pub fn forget_english_user_word(&mut self, text: &str) -> bool {
        self.user.forget_english_user_word(text)
    }

    /// 從清單選了常用語：照觸控鍵盤的 `text` 上屏（組字中先上屏組字），回傳快照給網頁插入。
    #[wasm_bindgen(js_name = commitText)]
    pub fn commit_text(&mut self, text: &str) -> WebSnapshot {
        self.session.text(&self.engine, &mut self.user, text).into()
    }

    /// 今天的日期 `YYYY-MM-DD`（使用者當地的日曆日）：網頁在第一次學習前、之後每次使用者操作前給
    /// （plan §2.4 殼的日期契約），學到的條目記成最後使用的日期；核心不讀系統時間（plan §1.3）。
    #[wasm_bindgen(js_name = setToday)]
    pub fn set_today(&mut self, today: &str) -> Result<(), JsError> {
        self.user
            .set_today(today.parse::<Date>().map_err(js_error)?);
        Ok(())
    }

    /// 這台裝置的 id（網頁產生一次、存 `localStorage`），匯出時寫進檔頭（plan §2.4）。
    #[wasm_bindgen(js_name = setDevice)]
    pub fn set_device(&mut self, id: &str) -> Result<(), JsError> {
        self.user.set_device(id).map_err(js_error)
    }

    /// 使用者詞與學習計數匯出成 `# qim-userdict v3` 的 TSV（plan §2.4），網頁存 `localStorage`、
    /// 下載成檔。
    #[wasm_bindgen(js_name = exportUserDict)]
    pub fn export_user_dict(&self) -> String {
        self.user.export()
    }

    /// 檢查 `text` 是不是匯入得了的檔，不動目前的資料；網頁請使用者確認覆蓋之前先檢查。回傳會略過的
    /// 列（不認得的種類），每條是「第 N 行：原因」。
    #[wasm_bindgen(js_name = checkUserDict)]
    pub fn check_user_dict(&self, text: &str) -> Result<Vec<String>, JsError> {
        let imported = UserModel::import(text, self.user.today()).map_err(js_error)?;
        Ok(warnings(&imported))
    }

    /// 讀匯出的檔，取代目前的使用者詞與學習計數；不合格式時整份不收、原本的不動。今天與這台裝置的
    /// id 留著（檔可能是別台匯出的）。回傳略過的列，同 [`WebSession::check_user_dict`]。
    #[wasm_bindgen(js_name = importUserDict)]
    pub fn import_user_dict(&mut self, text: &str) -> Result<Vec<String>, JsError> {
        let imported = UserModel::import(text, self.user.today()).map_err(js_error)?;
        let warnings = warnings(&imported);
        self.user.replace_learning(imported.user);
        Ok(warnings)
    }

    /// 先看匯入詞表 `text` 會加幾個詞、略過哪些列（在個人模型的複本上匯入），不動目前的資料；
    /// 網頁請使用者確認之前呼叫。整份讀不了時丟例外。`shortcuts_as_triggers` 同
    /// [`WebSession::import_word_list`]。
    #[wasm_bindgen(js_name = checkWordList)]
    pub fn check_word_list(
        &self,
        text: &str,
        shortcuts_as_triggers: bool,
    ) -> Result<WebWordListReport, JsError> {
        let report = self
            .user
            .clone()
            .import_word_list(&self.engine, text, shortcuts(shortcuts_as_triggers))
            .map_err(js_error)?;
        Ok(report.into())
    }

    /// 把詞表 `text` 的詞加進使用者詞（plan §2.4）：Rime `.dict.yaml`、Gboard 個人字典、一行一詞。
    /// 是加進去，不取代學習資料；整份讀不了時丟例外、什麼都不加。`shortcuts_as_triggers` 為真時
    /// Gboard 的快捷鍵設成觸發詞（預設不設，網頁的勾選框）。
    #[wasm_bindgen(js_name = importWordList)]
    pub fn import_word_list(
        &mut self,
        text: &str,
        shortcuts_as_triggers: bool,
    ) -> Result<WebWordListReport, JsError> {
        let report = self
            .user
            .import_word_list(&self.engine, text, shortcuts(shortcuts_as_triggers))
            .map_err(js_error)?;
        Ok(report.into())
    }

    /// 清掉全部使用者詞與學習計數；今天與這台裝置的 id 留著。
    #[wasm_bindgen(js_name = clearUserDict)]
    pub fn clear_user_dict(&mut self) {
        self.user.replace_learning(UserModel::default());
    }

    /// 簡體模式（M1 票 16）：候選、組字與上屏轉成簡體，學習仍記繁體。組字中切換也立刻
    /// 重列候選，回傳新的狀態給網頁畫。
    #[wasm_bindgen(js_name = setSimplified)]
    pub fn set_simplified(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.simplified = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 中文狀態的英文候選（M1 票 15）：整串輸入碼剛好是英文單字時排第 2 位，預設開。組字中
    /// 切換也立刻重列候選。
    #[wasm_bindgen(js_name = setEnglishCandidates)]
    pub fn set_english_candidates(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.english_candidates = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 繁體模式把簡體字排在同碼的繁體字之後（M3 票 19，brief §4.1），預設開。組字中切換也立刻
    /// 重列候選。
    #[wasm_bindgen(js_name = setSimplifiedAfterTraditional)]
    pub fn set_simplified_after_traditional(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.simplified_after_traditional = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// emoji 候選（M3 票 16）：整串的候選是 emoji 資料包裡的詞時，它的 emoji 接在它後面，預設開。
    /// 組字中切換也立刻重列候選。
    #[wasm_bindgen(js_name = setEmojiCandidates)]
    pub fn set_emoji_candidates(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.emoji_candidates = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 英文使用者詞（M4 票 14）：學打過的英文串，打過 2 次的在中文組字打完整串時排第 2，打過 3 次的
    /// 打開頭的字母也給，預設開。組字中切換也立刻重列候選。
    #[wasm_bindgen(js_name = setEnglishUserWords)]
    pub fn set_english_user_words(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.english_user_words = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 聯想（M3 票 13，brief §7）：上屏之後候選列給接下來可能要打的字詞，預設開。聯想顯示中
    /// 關掉也立刻收起。
    #[wasm_bindgen(js_name = setAssociation)]
    pub fn set_association(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.association = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 數字後的 `.` `:` 用半形（M1 票 26，brief §4.7），預設開。只影響下一個標點，組字不必重列。
    #[wasm_bindgen(js_name = setDigitSeparators)]
    pub fn set_digit_separators(&mut self, on: bool) {
        self.set_options(|options| options.digit_separators = on);
    }

    /// 組字中單按 Shift 的處理（`qim_core::ShiftSwitch`）：`rime`（預設）、`commit-text`、
    /// `inline-ascii`。不是按鍵，臨時英文中換了也不動組字。
    #[wasm_bindgen(js_name = setShiftSwitch)]
    pub fn set_shift_switch(&mut self, value: &str) -> Result<(), JsError> {
        let shift_switch = value.parse().map_err(js_error)?;
        self.set_options(|options| options.shift_switch = shift_switch);
        Ok(())
    }

    /// 按一個鍵（實體鍵盤）：種類與值照 [`Key::parse`]，例如 `("letter", "a")`、`("digit", "1")`、
    /// `("toggleAscii", "left")`、`("space")`。網頁的按鍵都經這裡。
    pub fn press(&mut self, kind: &str, value: Option<String>) -> Result<WebSnapshot, JsError> {
        let key = Key::parse(kind, value.as_deref()).map_err(js_error)?;
        Ok(self.handle(key).into())
    }

    /// 打出輸入碼裡的字元 `c`（`Key::for_input`：分隔記號送標點鍵，其餘送字母）；照輸入碼
    /// 重打一次組字時用（WebAssembly 的每鍵計時）。
    #[wasm_bindgen(js_name = inputChar)]
    pub fn input_char(&mut self, c: char) -> WebSnapshot {
        self.handle(Key::for_input(c)).into()
    }

    /// 全部候選，不分頁（候選展開面板用）；選字照位置呼叫 [`WebSession::select`]。
    #[wasm_bindgen(js_name = allCandidates)]
    pub fn all_candidates(&self) -> Vec<String> {
        self.session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    /// 給背景模型的重排要求（plan §2.1、§2.7）；沒有組字、移到某一段改字時是 `undefined`。
    #[wasm_bindgen(js_name = rerankRequest)]
    pub fn rerank_request(&mut self) -> Option<WebRerankRequest> {
        self.session
            .rerank_request(&self.engine)
            .map(|r| WebRerankRequest {
                generation: r.generation,
                context: r.context,
                scores: r.paths.iter().map(|p| p.score).collect(),
                paths: r.paths.into_iter().map(|p| p.text).collect(),
            })
    }

    /// 套用重排結果：`order` 是要求的路徑的序號，好的在前；排出的路徑到第 2 位，候選列沒有就插入。
    /// `generation` 過期時是 `undefined`。
    #[wasm_bindgen(js_name = applyRerank)]
    pub fn apply_rerank(&mut self, generation: u64, order: Vec<usize>) -> Option<WebSnapshot> {
        self.session
            .apply_rerank(&self.engine, generation, &order)
            .map(Into::into)
    }

    /// 正在用的停鍵層模型的識別（`Session::set_reranker_id`，plan §2.7）：模式加上模型資料包；關掉重排或
    /// 沒有模型時是 `undefined`。換了就清掉停鍵結果快取、給舊模型的要求作廢，`undefined` 時不留快取。
    #[wasm_bindgen(js_name = setRerankerId)]
    pub fn set_reranker_id(&mut self, reranker_id: Option<String>) {
        self.session.set_reranker_id(reranker_id.as_deref());
    }

    /// debug 面板（plan §3.1，M4 票 01）：核心為什麼這樣排（`Session::inspect`）。網頁只在面板開著時呼叫。
    pub fn inspect(&self) -> WebInspection {
        self.session.inspect(&self.engine, &self.user).into()
    }

    /// 選全部候選的第 `index` 個（目前頁第 i 個是 `page × 9 + i`），例如點候選列。
    pub fn select(&mut self, index: usize) -> WebSnapshot {
        self.session
            .select(&self.engine, &mut self.user, index)
            .into()
    }

    /// 移到組字的第 `segment` 段，例如點組字裡的那一段；分段數是回到整串。
    #[wasm_bindgen(js_name = focusSegment)]
    pub fn focus_segment(&mut self, segment: usize) -> WebSnapshot {
        self.session
            .focus_segment(&self.engine, &mut self.user, segment)
            .into()
    }

    /// 點一下 `layout` 目前頁第 `index` 個鍵（點在畫布座標 `(x, y)`，`index` 是 [`WebLayout::hit`] 的結果），
    /// 見 [`Keyboard::tap`]；字母鍵依點擊位置送各字母的機率（`Key::Weighted`，M4 票 06）。
    /// `mm_per_unit` 是畫布一單位幾毫米，`quick_repeat` 是同一個鍵剛點過（網頁量時間）。
    pub fn tap(
        &mut self,
        layout: &mut WebLayout,
        index: usize,
        quick_repeat: bool,
        x: f64,
        y: f64,
        mm_per_unit: f64,
    ) -> Result<WebSnapshot, JsError> {
        let touch = Touch { x, y, mm_per_unit };
        let output = layout
            .keyboard
            .tap(index, self.session.composing(), quick_repeat, Some(touch))
            .map_err(js_error)?;
        self.send(output)
    }

    /// 長按 `layout` 目前頁第 `index` 個鍵、選了長按清單第 `choice` 個（0 是預設）。
    #[wasm_bindgen(js_name = longPress)]
    pub fn long_press(
        &mut self,
        layout: &mut WebLayout,
        index: usize,
        choice: usize,
    ) -> Result<WebSnapshot, JsError> {
        let output = layout
            .keyboard
            .long_press(index, choice)
            .map_err(js_error)?;
        self.send(output)
    }
}

impl WebSession {
    /// 改目前選項的一項再交給核心。
    fn set_options(&mut self, change: impl FnOnce(&mut Options)) {
        let mut options = self.session.options();
        change(&mut options);
        self.session.set_options(options);
    }

    fn handle(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 觸控鍵盤的輸入交給核心：`Output` 轉成 `Key`，文字經 `Session::text`。結果帶著輸入的種類與
    /// 字元，核心沒吃下時網頁照它做。
    fn send(&mut self, output: Output) -> Result<WebSnapshot, JsError> {
        let snapshot = match &output {
            Output::Text(text) => self.session.text(&self.engine, &mut self.user, text),
            Output::Letter(c) => self.handle(Key::Letter(*c)),
            // `qim-layout` 給的一定收得下（字母、不重複、正數）；收不下是 bug，照例外丟給網頁。
            Output::Weighted(weights) => {
                let weights = Weights::new(weights.clone()).map_err(js_error)?;
                self.handle(Key::Weighted(weights))
            }
            Output::Punct(c) => self.handle(Key::Punct(*c)),
            Output::Space => self.handle(Key::Space),
            Output::Backspace => self.handle(Key::Backspace),
            Output::Enter => self.handle(Key::Enter),
            Output::ToggleAscii => self.handle(Key::ToggleAscii(ShiftKey::Right)),
            Output::Modifier => self.handle(Key::Modifier),
        };
        let mut snapshot = WebSnapshot::from(snapshot);
        snapshot.action = Some(output.kind().into());
        snapshot.character = output.character().map(String::from);
        Ok(snapshot)
    }
}

/// `Snapshot` 給 JS 的樣子：攤平成網頁用得到的欄位，其他欄位等網頁用到時再加。JS 端叫 `Snapshot`。
#[wasm_bindgen(js_name = Snapshot, getter_with_clone)]
pub struct WebSnapshot {
    /// 為假時網頁不攔截這個鍵，交給瀏覽器。
    #[wasm_bindgen(readonly)]
    pub handled: bool,
    /// 組字的整串輸入碼，含已選定的分段的碼；「回報」用。
    #[wasm_bindgen(readonly)]
    pub input: String,
    /// 已選定的分段接起來的字詞；「回報」用。
    #[wasm_bindgen(readonly)]
    pub selected: String,
    /// 組字的各分段，依輸入碼的順序。
    #[wasm_bindgen(readonly)]
    pub segments: Vec<WebSegment>,
    /// 目前分段在 `segments` 的位置；等於 `segments.length` 時候選列整串（沒有目前分段）。
    #[wasm_bindgen(readonly)]
    pub focus: usize,
    /// 目前頁的候選，經過後處理（簡體模式是簡體）。
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<String>,
    /// 目前頁的候選的繁體原文，和 `candidates` 一一對應；字根提示與「回報」用。
    #[wasm_bindgen(readonly, js_name = candidatesOriginal)]
    pub candidates_original: Vec<String>,
    /// 目前頁，從 0 起。
    #[wasm_bindgen(readonly)]
    pub page: usize,
    #[wasm_bindgen(readonly)]
    pub commit: Option<String>,
    /// 觸控鍵盤交給核心的輸入的種類（`qim_layout::Output::kind`），長按時是選到的那個；核心沒吃下時
    /// 網頁照它做宿主的預設動作。實體鍵盤的鍵是 `None`。
    #[wasm_bindgen(readonly)]
    pub action: Option<String>,
    /// `action` 是 `letter` 或 `punct` 時的字元；核心沒吃下時網頁照打它。
    #[wasm_bindgen(readonly)]
    pub character: Option<String>,
    /// `commit` 的繁體原文；「回報」用。
    #[wasm_bindgen(readonly, js_name = commitOriginal)]
    pub commit_original: Option<String>,
    /// 英文直通（單按 Shift 切換）；候選列旁顯示目前狀態。
    #[wasm_bindgen(readonly)]
    pub ascii: bool,
    /// 每次候選變動加一；和上次要求重排時不同，才再要求（plan §2.7）。
    #[wasm_bindgen(readonly)]
    pub generation: u64,
}

/// 給背景模型的重排要求（`qim_core::RerankRequest`）。JS 端叫 `RerankRequest`。
#[wasm_bindgen(js_name = RerankRequest, getter_with_clone)]
pub struct WebRerankRequest {
    #[wasm_bindgen(readonly)]
    pub generation: u64,
    /// 路徑前面的字：前文（簡體模式下逐字轉回繁體）接上選定的分段，最多 50 字。
    #[wasm_bindgen(readonly)]
    pub context: String,
    /// lattice 的前 9 條整句路徑的繁體原文，依路徑分由高到低；簡體模式下模型也看繁體。路徑分另放
    /// `scores`、不帶分段：JS 端的模型只要文字與路徑分，兩個平的陣列不必為每條路徑另建 JS 物件。
    #[wasm_bindgen(readonly)]
    pub paths: Vec<String>,
    /// 各路徑的同步層路徑分，和 `paths` 同順序（神經模型的 `路徑分 + λ·神經分` 用）。
    #[wasm_bindgen(readonly)]
    pub scores: Vec<f64>,
}

/// 停鍵層的神經重排（`qim_neural::Reranker`，plan §2.7）：網頁在 Web Worker 裡建一個，主執行緒的
/// `Session` 不碰它。JS 端叫 `Reranker`。
#[wasm_bindgen(js_name = Reranker)]
pub struct WebReranker(qim_neural::Reranker);

#[wasm_bindgen(js_class = Reranker)]
impl WebReranker {
    /// 由模型資料包（種類 `model`）建立：核心核對 manifest 與位元組（`ModelPack`），權重與字表交給
    /// `qim-neural`。資料包用完由 JS 端釋放。
    #[wasm_bindgen(constructor)]
    pub fn new(pack: &WebPack) -> Result<WebReranker, JsError> {
        let model: ModelPack = pack.open()?;
        qim_neural::Reranker::new(
            &model.weights,
            &model.vocab,
            model.unknown_chars,
            model.lambda,
        )
        .map(Self)
        .map_err(js_error)
    }

    /// 重排要求（[`WebRerankRequest`]）的前文、路徑與路徑分 → 路徑的序號（好的在前，交給
    /// [`WebSession::apply_rerank`]）與各路徑的總分（debug 面板）。`paths` 與 `scores` 長度不同時丟錯。
    pub fn rank(
        &self,
        context: &str,
        paths: Vec<String>,
        scores: Vec<f64>,
    ) -> Result<WebRanking, JsError> {
        if paths.len() != scores.len() {
            return Err(JsError::new(&format!(
                "路徑 {} 條、路徑分 {} 個",
                paths.len(),
                scores.len()
            )));
        }
        let paths: Vec<(&str, f64)> = paths.iter().map(String::as_str).zip(scores).collect();
        let ranking = self.0.rank_scored(context, &paths).map_err(js_error)?;
        Ok(WebRanking {
            order: ranking.order,
            totals: ranking.totals,
        })
    }
}

/// 停鍵層模型的結果（`qim_neural::Ranking`）。JS 端叫 `Ranking`。
#[wasm_bindgen(js_name = Ranking, getter_with_clone)]
pub struct WebRanking {
    /// 路徑的序號，好的在前。
    #[wasm_bindgen(readonly)]
    pub order: Vec<usize>,
    /// 各路徑的 `路徑分 + λ·神經分`，和要求的路徑同順序。
    #[wasm_bindgen(readonly)]
    pub totals: Vec<f64>,
}

/// debug 面板的資料（`qim_core::Inspection`）。JS 端叫 `Inspection`。
#[wasm_bindgen(js_name = Inspection, getter_with_clone)]
pub struct WebInspection {
    /// 核心記著的前文：`known`、`unknown`（不知道前文）、`incognito`（無痕不讀）。
    #[wasm_bindgen(readonly, js_name = contextState)]
    pub context_state: String,
    /// `contextState` 是 `known` 時的前文，其餘是空字串。
    #[wasm_bindgen(readonly)]
    pub context: String,
    /// 開頭的字接的前一字：`clauseStart`（句首）、`char`（`precedingChar`）或 `unigram`。
    #[wasm_bindgen(readonly, js_name = precedingKind)]
    pub preceding_kind: String,
    #[wasm_bindgen(readonly, js_name = precedingChar)]
    pub preceding_char: Option<String>,
    /// 開頭的一段接的前一個字詞（個人 bigram）。
    #[wasm_bindgen(readonly, js_name = precedingWord)]
    pub preceding_word: Option<String>,
    /// 組字的各分段。
    #[wasm_bindgen(readonly)]
    pub segments: Vec<WebSegment>,
    /// 全部候選，依目前的順序。
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<WebInspectedCandidate>,
    /// 最近一個鍵是觸控字母鍵時的各字母（M4 票 06），由機率高到低；其他鍵是空的。
    #[wasm_bindgen(readonly, js_name = touchLetters)]
    pub touch_letters: Vec<String>,
    /// `touchLetters` 各字母的機率，和為 1。
    #[wasm_bindgen(readonly, js_name = touchProbabilities)]
    pub touch_probabilities: Vec<f32>,
    /// 候選列的停鍵重排來自快取（M4 票 03）：沒等停鍵、沒跑模型。
    #[wasm_bindgen(readonly, js_name = rerankCached)]
    pub rerank_cached: bool,
}

/// debug 面板的一個候選（`qim_core::InspectedCandidate`）。JS 端叫 `InspectedCandidate`。
#[wasm_bindgen(js_name = InspectedCandidate, getter_with_clone)]
#[derive(Clone)]
pub struct WebInspectedCandidate {
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// `word`、`english`、`emoji`、`englishUserWord`、`association`。
    #[wasm_bindgen(readonly)]
    pub kind: String,
    #[wasm_bindgen(readonly)]
    pub segments: Vec<WebInspectedSegment>,
    /// 路徑分；不在 lattice 裡的候選沒有。
    #[wasm_bindgen(readonly)]
    pub score: Option<f64>,
    /// 路徑分裡學到的加分。
    #[wasm_bindgen(readonly)]
    pub learning: Option<f64>,
    /// 排定它位置的排位規則（1–5）。
    #[wasm_bindgen(readonly)]
    pub rule: Option<u8>,
    /// 停鍵重排放到這裡的：重排要求的第幾條路徑。
    #[wasm_bindgen(readonly, js_name = rerankPath)]
    pub rerank_path: Option<usize>,
    /// 停鍵重排插入的（候選列原本沒有）；從後面移上來的是 `false`。
    #[wasm_bindgen(readonly, js_name = rerankInserted)]
    pub rerank_inserted: bool,
}

/// debug 面板的候選的一段（`qim_core::InspectedSegment`）。JS 端叫 `InspectedSegment`。
#[wasm_bindgen(js_name = InspectedSegment, getter_with_clone)]
#[derive(Clone)]
pub struct WebInspectedSegment {
    #[wasm_bindgen(readonly)]
    pub code: String,
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// `codeTable`、`lexicon`、`userWord`。
    #[wasm_bindgen(readonly)]
    pub source: String,
    #[wasm_bindgen(readonly)]
    pub learned: bool,
    #[wasm_bindgen(readonly, js_name = personalBigram)]
    pub personal_bigram: bool,
    /// 碼有萬用鍵時實際對到的碼；沒有萬用鍵時沒有。
    #[wasm_bindgen(readonly, js_name = matchedCode)]
    pub matched_code: Option<String>,
}

impl From<Inspection> for WebInspection {
    fn from(i: Inspection) -> Self {
        let (context_state, context) = match i.context {
            ContextState::Known(text) => ("known", text),
            ContextState::Unknown => ("unknown", String::new()),
            ContextState::Incognito => ("incognito", String::new()),
        };
        let (preceding_kind, preceding_char) = match i.preceding_char {
            PrecedingChar::ClauseStart => ("clauseStart", None),
            PrecedingChar::Char(c) => ("char", Some(String::from(c))),
            PrecedingChar::Unigram => ("unigram", None),
        };
        let touch = i.touch_weights.as_ref().map_or(&[][..], Weights::entries);
        Self {
            touch_letters: touch.iter().map(|&(c, _)| String::from(c)).collect(),
            touch_probabilities: touch.iter().map(|&(_, p)| p).collect(),
            rerank_cached: i.rerank_cached,
            context_state: context_state.into(),
            context,
            preceding_kind: preceding_kind.into(),
            preceding_char,
            preceding_word: i.preceding_word,
            segments: i.segments.into_iter().map(WebSegment::from).collect(),
            candidates: i
                .candidates
                .into_iter()
                .map(|c| WebInspectedCandidate {
                    text: c.text,
                    kind: match c.kind {
                        CandidateKind::Word => "word",
                        CandidateKind::English => "english",
                        CandidateKind::Emoji => "emoji",
                        CandidateKind::EnglishUserWord => "englishUserWord",
                        CandidateKind::Association => "association",
                    }
                    .into(),
                    segments: c
                        .segments
                        .into_iter()
                        .map(|s| WebInspectedSegment {
                            code: s.code,
                            text: s.text,
                            source: match s.source {
                                SegmentSource::CodeTable => "codeTable",
                                SegmentSource::Lexicon => "lexicon",
                                SegmentSource::UserWord => "userWord",
                            }
                            .into(),
                            learned: s.learned,
                            personal_bigram: s.personal_bigram,
                            matched_code: s.matched_code,
                        })
                        .collect(),
                    score: c.score,
                    learning: c.learning,
                    rule: c.rule,
                    rerank_path: c.rerank.map(|r| r.path),
                    rerank_inserted: c.rerank.is_some_and(|r| r.inserted),
                })
                .collect(),
        }
    }
}

impl From<Snapshot> for WebSnapshot {
    fn from(s: Snapshot) -> Self {
        Self {
            handled: s.handled,
            input: s.composition.input.clone(),
            selected: s.composition.selected_text(),
            focus: s.composition.focus,
            segments: s
                .composition
                .segments
                .into_iter()
                .map(WebSegment::from)
                .collect(),
            candidates_original: s.candidates.iter().map(|c| c.original.clone()).collect(),
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            page: s.page,
            commit: s.commit,
            action: None,
            character: None,
            commit_original: s.commit_original,
            ascii: s.ascii,
            generation: s.generation,
        }
    }
}

/// 核心留著的游標前文字最多幾個字（`qim_core::CONTEXT_LIMIT`）：網頁給前文時只取這麼多。
#[wasm_bindgen(js_name = contextLimit)]
pub fn context_limit() -> usize {
    qim_core::CONTEXT_LIMIT
}

/// 候選每頁的個數（`qim_core::PAGE_SIZE`）：網頁把目前頁的位置換算成 [`WebSession::select`] 的位置。
#[wasm_bindgen(js_name = pageSize)]
pub fn page_size() -> usize {
    qim_core::PAGE_SIZE
}

/// 組字的一段（`qim_core::Segment`）。JS 端叫 `Segment`。
#[wasm_bindgen(js_name = Segment, getter_with_clone)]
#[derive(Clone)]
pub struct WebSegment {
    #[wasm_bindgen(readonly)]
    pub code: String,
    /// `code` 的字根；組字裡沒選定的分段顯示它（brief §4.3）。
    #[wasm_bindgen(readonly)]
    pub radicals: String,
    /// 沒有路徑蓋得滿剩下的輸入碼時是空的。
    #[wasm_bindgen(readonly)]
    pub text: String,
    #[wasm_bindgen(readonly)]
    pub selected: bool,
    /// 臨時英文（組字中單按留碼的 Shift）：`text` 是原樣的英文，顯示它而不是字根。
    #[wasm_bindgen(readonly)]
    pub raw: bool,
}

impl From<qim_core::Segment> for WebSegment {
    fn from(segment: qim_core::Segment) -> Self {
        Self {
            code: segment.code,
            radicals: segment.radicals,
            text: segment.text,
            selected: segment.selected,
            raw: segment.raw,
        }
    }
}

/// 字根提示：方案指名的字根提示資料包，查候選每個字的倉頡碼與字根（plan §2.5）。JS 端叫 `RadicalHints`。
#[wasm_bindgen(js_name = RadicalHints)]
pub struct WebRadicalHints {
    hints: RadicalHints,
    /// 建立時的方案，換字根用。
    scheme: Scheme,
}

#[wasm_bindgen(js_class = RadicalHints)]
impl WebRadicalHints {
    /// 由字根提示資料包建立；`session` 的方案要指名字根提示（可以是另一份，plan §4.6）。
    #[wasm_bindgen(constructor)]
    pub fn new(pack: &WebPack, session: &WebSession) -> Result<WebRadicalHints, JsError> {
        let scheme = session.engine.scheme().clone();
        let pack: RadicalHintPack = pack.open()?;
        let hints = RadicalHints::for_scheme(pack, &scheme).map_err(js_error)?;
        Ok(Self { hints, scheme })
    }

    /// 候選 `text` 每個字的提示；`version` 是 `cangjie5` 或 `cangjie3`。
    pub fn hint(&self, text: &str, version: &str) -> Result<Vec<WebRadicalHint>, JsError> {
        let version: CangjieVersion = version.parse().map_err(js_error)?;
        Ok(self
            .hints
            .hint(&self.scheme, text, version)
            .into_iter()
            .map(|hint| WebRadicalHint {
                text: hint.text.to_string(),
                codes: hint.codes,
                radicals: hint.radicals,
            })
            .collect())
    }
}

/// 詞表匯入的結果。JS 端叫 `WordListReport`。
#[wasm_bindgen(js_name = WordListReport, getter_with_clone)]
pub struct WebWordListReport {
    /// 認出的格式，給人看的名字。
    #[wasm_bindgen(readonly)]
    pub format: String,
    /// 加進使用者詞的詞數。
    #[wasm_bindgen(readonly)]
    pub added: usize,
    /// 沒加的列，一列一句（「第 n 行「詞」：原因」）。
    #[wasm_bindgen(readonly)]
    pub skipped: Vec<String>,
    /// 快捷鍵能設成觸發詞的列，一列一句；會搶走首選的寫出搶走誰。
    #[wasm_bindgen(readonly, js_name = triggerShortcuts)]
    pub trigger_shortcuts: Vec<String>,
    /// `trigger_shortcuts` 裡設了會搶走首選的列（同樣的句子），網頁拿來標出。
    #[wasm_bindgen(readonly, js_name = displacingShortcuts)]
    pub displacing_shortcuts: Vec<String>,
    /// 加了、但快捷鍵不是方案的鍵而不能當觸發詞的列，一列一句。
    #[wasm_bindgen(readonly, js_name = unusedShortcuts)]
    pub unused_shortcuts: Vec<String>,
}

/// 網頁勾選框的值轉成核心的選項。
fn shortcuts(as_triggers: bool) -> Shortcuts {
    if as_triggers {
        Shortcuts::AsTriggers
    } else {
        Shortcuts::Ignore
    }
}

impl From<WordListReport> for WebWordListReport {
    fn from(report: WordListReport) -> Self {
        Self {
            format: report.format.to_string(),
            added: report.added,
            skipped: report.skipped.iter().map(ToString::to_string).collect(),
            trigger_shortcuts: report
                .trigger_shortcuts
                .iter()
                .map(ToString::to_string)
                .collect(),
            displacing_shortcuts: report
                .trigger_shortcuts
                .iter()
                .filter(|t| t.displaces.is_some())
                .map(ToString::to_string)
                .collect(),
            unused_shortcuts: report
                .unused_shortcuts
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

/// 一條常用語。JS 端叫 `Phrase`。
#[wasm_bindgen(js_name = Phrase, getter_with_clone)]
pub struct WebPhrase {
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// 觸發詞；沒有時是空字串。
    #[wasm_bindgen(readonly)]
    pub trigger: String,
}

/// 候選裡一個字的字根提示。JS 端叫 `RadicalHint`。
#[wasm_bindgen(js_name = RadicalHint, getter_with_clone)]
pub struct WebRadicalHint {
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// 倉頡碼，沒有時是空的。
    #[wasm_bindgen(readonly)]
    pub codes: Vec<String>,
    /// 每個碼的字根，和 `codes` 一一對應。
    #[wasm_bindgen(readonly)]
    pub radicals: Vec<String>,
}

/// 觸控鍵盤（[`Keyboard`]）：網頁 fetch 的佈局檔讀進 `qim-layout`，網頁只照 [`WebLayout::keys`] 畫、
/// 把點擊位置交給 [`WebLayout::hit`]。JS 端叫 `Layout`。
#[wasm_bindgen(js_name = Layout)]
pub struct WebLayout {
    keyboard: Keyboard,
    /// 驗證與選版本的警告，給網頁印在 console。
    warnings: Vec<String>,
}

/// 網頁傳來的裝置（`phone`、`tablet`）與方向（`portrait`、`landscape`）。
fn when(device: &str, orientation: &str) -> Result<When, JsError> {
    Ok(When {
        device: device.parse().map_err(js_error)?,
        orientation: orientation.parse().map_err(js_error)?,
    })
}

#[wasm_bindgen(js_class = Layout)]
impl WebLayout {
    /// 讀佈局檔並依裝置與方向選版本；有錯誤（plan §2.6）時失敗。
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str, device: &str, orientation: &str) -> Result<WebLayout, JsError> {
        let when = when(device, orientation)?;
        let (layout, warnings) = qim_layout::validate(json, Target::default())
            .into_layout()
            .map_err(js_error)?;
        let (keyboard, warning) = Keyboard::new(layout, when);
        Ok(Self {
            keyboard,
            warnings: warnings
                .iter()
                .chain(&warning)
                .map(ToString::to_string)
                .collect(),
        })
    }

    /// 依裝置與方向重選佈局版本（轉向時叫），見 [`Keyboard::select`]；沒有相符的版本時回傳警告。
    pub fn select(&mut self, device: &str, orientation: &str) -> Result<Option<String>, JsError> {
        let warning = self.keyboard.select(when(device, orientation)?);
        Ok(warning.map(|w| w.to_string()))
    }

    /// 換成同一個鍵盤的另一份佈局檔（數字列開關）時接手 `old` 的 Shift 與目前頁
    /// （[`Keyboard::keep_state_of`]）。
    #[wasm_bindgen(js_name = keepStateOf)]
    pub fn keep_state_of(&mut self, old: &WebLayout) {
        self.keyboard.keep_state_of(&old.keyboard);
    }

    /// 觸控 Shift 的狀態：`off`、`once`（下一個字母大寫）、`locked`（鎖定大寫）；網頁依它畫 Shift 鍵、
    /// 狀態變了就重畫標籤。
    #[wasm_bindgen(getter)]
    pub fn shift(&self) -> String {
        match self.keyboard.shift() {
            qim_layout::Shift::Off => "off",
            qim_layout::Shift::Once => "once",
            qim_layout::Shift::Locked => "locked",
        }
        .into()
    }

    /// 目前頁的 id；切頁鍵按下後網頁比對它決定要不要重畫。
    #[wasm_bindgen(getter, js_name = pageId)]
    pub fn page_id(&self) -> String {
        self.keyboard.page().id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    /// 設計尺寸的寬，按鍵座標以它為準。
    #[wasm_bindgen(getter, js_name = canvasWidth)]
    pub fn canvas_width(&self) -> f64 {
        self.keyboard.variant().canvas.width
    }

    #[wasm_bindgen(getter, js_name = canvasHeight)]
    pub fn canvas_height(&self) -> f64 {
        self.keyboard.variant().canvas.height
    }

    /// 目前頁的按鍵：標籤省略時用 `session` 方案的字根，組字中照 `composing`，Shift 開著時字母鍵
    /// 顯示大寫字母。
    pub fn keys(&self, session: &WebSession) -> Vec<WebLayoutKey> {
        let radicals = &session.engine.scheme().radicals;
        let state = self.keyboard.key_state(session.session.composing());
        self.keyboard
            .page()
            .keys
            .iter()
            .map(|key| {
                let label = key.display_label(state, |c| radicals.get(&c).map(String::as_str));
                let style = key.style.clone().unwrap_or_default();
                let [label_offset_x, label_offset_y] = style.label_offset.unwrap_or([0.0, 0.0]);
                WebLayoutKey {
                    id: key.id.clone(),
                    x: key.frame.x,
                    y: key.frame.y,
                    w: key.frame.w,
                    h: key.frame.h,
                    rotate: key.rotate,
                    z: key.z,
                    radius: key.shape.corner_radius(),
                    a11y_label: key.accessibility_label(state, &label),
                    label,
                    hint: key.display_hint(state).map(str::to_owned),
                    is_shift: key.tap_action(state) == Action::Shift,
                    fill_light: style.fill.as_ref().map(|c| c.light.clone()),
                    fill_dark: style.fill.map(|c| c.dark),
                    label_color_light: style.label_color.as_ref().map(|c| c.light.clone()),
                    label_color_dark: style.label_color.map(|c| c.dark),
                    opacity: style.opacity,
                    label_size: key.display_label_size(state),
                    label_offset_x,
                    label_offset_y,
                    preview: key.shows_preview(state),
                    long_press: key.long_press.iter().map(Action::spoken).collect(),
                }
            })
            .collect()
    }

    /// 點在畫布座標 `(x, y)` 時按到的鍵在 [`WebLayout::keys`] 的位置。
    pub fn hit(&self, x: f64, y: f64) -> Option<usize> {
        self.keyboard.page().hit(x, y)
    }
}

/// 佈局的一個鍵給 JS 畫的樣子，座標是設計尺寸；顏色淺色與深色都給，由網頁依目前的配色選。
/// JS 端叫 `LayoutKey`。
#[wasm_bindgen(js_name = LayoutKey, getter_with_clone)]
pub struct WebLayoutKey {
    #[wasm_bindgen(readonly)]
    pub id: String,
    #[wasm_bindgen(readonly)]
    pub x: f64,
    #[wasm_bindgen(readonly)]
    pub y: f64,
    #[wasm_bindgen(readonly)]
    pub w: f64,
    #[wasm_bindgen(readonly)]
    pub h: f64,
    /// 以中心順時針旋轉的角度（度）。
    #[wasm_bindgen(readonly)]
    pub rotate: f64,
    #[wasm_bindgen(readonly)]
    pub z: i32,
    /// 圓角半徑，設計尺寸。
    #[wasm_bindgen(readonly)]
    pub radius: f64,
    /// 主標籤，已照 [`qim_layout::Key::display_label`] 補上字根。
    #[wasm_bindgen(readonly)]
    pub label: String,
    /// 角標（[`qim_layout::Key::display_hint`]），Shift 開著時字母鍵沒有。
    #[wasm_bindgen(readonly)]
    pub hint: Option<String>,
    /// 現在點下去是不是 Shift（組字中 ⇧ 是分隔鍵，不是）；網頁依 [`WebLayout::shift`] 畫它的狀態。
    #[wasm_bindgen(readonly, js_name = isShift)]
    pub is_shift: bool,
    #[wasm_bindgen(readonly, js_name = a11yLabel)]
    pub a11y_label: String,
    #[wasm_bindgen(readonly, js_name = fillLight)]
    pub fill_light: Option<String>,
    #[wasm_bindgen(readonly, js_name = fillDark)]
    pub fill_dark: Option<String>,
    #[wasm_bindgen(readonly, js_name = labelColorLight)]
    pub label_color_light: Option<String>,
    #[wasm_bindgen(readonly, js_name = labelColorDark)]
    pub label_color_dark: Option<String>,
    #[wasm_bindgen(readonly)]
    pub opacity: Option<f64>,
    /// 主標籤字級，設計尺寸。
    #[wasm_bindgen(readonly, js_name = labelSize)]
    pub label_size: Option<f64>,
    #[wasm_bindgen(readonly, js_name = labelOffsetX)]
    pub label_offset_x: f64,
    #[wasm_bindgen(readonly, js_name = labelOffsetY)]
    pub label_offset_y: f64,
    /// 按住時要不要顯示放大預覽（[`qim_layout::Key::shows_preview`]），組字中照 `composing`。
    #[wasm_bindgen(readonly)]
    pub preview: bool,
    /// 長按清單各選項的標籤（[`Action::spoken`]），依序排、第一個是預設；空的就沒有長按。
    #[wasm_bindgen(readonly, js_name = longPress)]
    pub long_press: Vec<String>,
}
