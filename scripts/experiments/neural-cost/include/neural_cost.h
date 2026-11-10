// neural-cost 靜態庫的 C 介面（src/lib.rs）。iOS 鍵盤擴充經 bridging header 引用。
#pragma once
#include <stdint.h>

// 量一個模型檔：載入、先跑一次、上下文 0 / 50 / 200 字各跑 iterations 次重排，threads 條執行緒。
// 回 UTF-8 的多行結果（錯誤以 "error:" 開頭），用完交給 nc_free。
char *nc_run(const char *model_path, uint32_t iterations, uint32_t threads);
void nc_free(char *s);
