# TODO

## 當前版本：v0.1.1

### 字型

- [x] ~~Noto Sans CJK TC 替換為 Fusion Pixel Font 12px~~
- [ ] 測試 Fusion Pixel 12px 繁中缺字情況（覆蓋率 92% Big5），需要 Noto Sans CJK TC 作為 fallback
- [ ] 評估 [GNU Unifont 16px](https://unifoundry.com/unifont/index.html) 作為大字標題用途（OFL 雙授權）

### UI 改善

- [ ] egui prototype 的兩欄佈局需正確使用 `ui.available_width()` 計算元件寬度
- [ ] 存檔/讀檔 UI（主選單 Load 按鈕 + 遊戲中存檔按鈕）
- [ ] 勝利/失敗畫面（三大獎全拿 → 勝利、破產 → 失敗）
- [ ] 結局判定系統（多種結局條件）

### 系統補完

- [ ] 服裝穿戴影響通告成功率（equipped outfit 的 image tag 加成應用於 gig success 計算）
- [ ] 迷你遊戲 auto-resolve 整合到通告完成流程
- [ ] 敘事腳本依行事曆觸發（narrative engine 已有引擎但未接入 advance_week）
- [ ] 合約管理（續約/重新談判/到期）
- [ ] 0 個藝人時「推進一週」的行為確認（目前允許，可能需要提示玩家招募）

### 遊戲內容

- [ ] 更多訓練類型（儀態、口才、創作訓練）
- [ ] 更多打工種類
- [ ] 更多通告（不同類型和難度梯度）
- [ ] 更多危機事件
- [ ] 更多招募候選人
- [ ] 更多服裝
- [ ] 劇情腳本內容

### Phase 5B：PC-98 像素風 UI

- [ ] 用 Bevy 原生 UI 替換 egui（正式美術）
- [ ] 藝人立繪 / 肖像
- [ ] 背景圖（辦公室場景隨等級變化）
- [ ] UI 元件的像素風格設計
- [ ] 畫面轉場動畫

### 音效

- [ ] BGM 系統（主選單、遊戲中、事件）
- [ ] SE 音效（按鈕、通告完成、獎項、危機）

### 發布準備

- [ ] 遊戲平衡調整（數值、AI 對手強度、經濟曲線）
- [ ] 多結局測試
- [ ] 跨平台編譯測試（Windows、macOS、Linux）
