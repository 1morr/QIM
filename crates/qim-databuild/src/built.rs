//! 一次建置建出的資料包清單：`pipeline` 依它寫檔、印 sha256，`verify` 依它比對兩次建置與解開。

use qim_core::{Pack, PackError, PackKind};

use crate::sources::sha256_hex;

/// 建出的一份資料包。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltPack {
    /// 給人看的種類名（`速成`、`詞庫`…），印在建置輸出與驗證報告。
    pub name: &'static str,
    pub id: String,
    /// manifest 的種類（[`Pack::KIND`]）。
    pub kind: PackKind,
    /// 這種資料包的魔數（[`Pack::MAGIC`]）；[`find`] 依它取出資料包，不看 `bytes` 的內容。
    pub magic: [u8; 4],
    pub bytes: Vec<u8>,
}

impl BuiltPack {
    pub fn new<P: Pack>(name: &'static str, pack: &P) -> Result<Self, PackError> {
        Ok(Self {
            name,
            id: pack.id().into(),
            kind: P::KIND,
            magic: P::MAGIC,
            bytes: pack.to_bytes()?,
        })
    }

    pub fn sha256(&self) -> String {
        sha256_hex(&self.bytes)
    }
}

/// 清單裡 `P` 那種資料包的第一份。詞庫有兩份（預設的在前、粵典的在後），其餘每種一份；要後面那份
/// 用 [`find_id`]。
pub fn find<P: Pack>(packs: &[BuiltPack]) -> Result<&BuiltPack, String> {
    packs
        .iter()
        .find(|p| p.magic == P::MAGIC)
        .ok_or_else(|| format!("沒有魔數 {} 的資料包", String::from_utf8_lossy(&P::MAGIC)))
}

/// 清單裡 id 是 `id` 的資料包。
pub fn find_id<'a>(packs: &'a [BuiltPack], id: &str) -> Result<&'a BuiltPack, String> {
    packs
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("沒有 id {id} 的資料包"))
}

/// 取出清單裡 `P` 那種資料包並解開。
pub fn decode<P: Pack>(packs: &[BuiltPack]) -> Result<P, String> {
    let pack = find::<P>(packs)?;
    P::from_bytes(&pack.bytes).map_err(|e| format!("{}資料包：{e}", pack.name))
}
