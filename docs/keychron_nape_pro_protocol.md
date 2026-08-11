# Keychron Nape Pro & Keychron Launcher HID プロトコル仕様書

本ドキュメントは、Keychron 公式 Web アプリ (`https://launcher.keychron.com/` バンドル `main.196de488e6ee7c68.js`) の JavaScript ソースコード解析・逆コンパイルによって解読された、**Keychron Nape Pro トラックボールとの HID 通信プロトコル仕様**です。

---

## 1. デバイス識別要件

| 項目 | 値 | 備考 |
| :--- | :--- | :--- |
| **Vendor ID (VID)** | `0x3434` (`13364`) | Keychron |
| **Product ID (PID)** | `0x0440` (`1088`) | Keychron Nape Pro |
| **Raw HID Usage Page** | `0xFF60` | VIA Raw HID Endpoint |
| **Raw HID Usage** | `0x0061` | VIA Raw HID Endpoint |

---

## 2. Keychron 公式 Vendor コマンド定数一覧

Keychron ランチャーソースコード内で定義されているトップレベルのコマンド ID（Report ID `0x00` の次のバイトに配置）です。

```typescript
enum KeychronVendorCommand {
  KC_GET_PROTOCOL_VERSION = 160, // 0xA0
  KC_FIRMWARE_VERSION     = 161, // 0xA1: ファームウェアビルド日時文字列の取得
  KC_GET_SUPPORT_FEATURE  = 162, // 0xA2: サポート機能フラグの取得
  KC_GET_CURRENT_LAYER    = 163, // 0xA3: 現在のレイヤー状態取得 (Read-Only)
  KC_MISC_CMD_GROUP       = 167, // 0xA7: Keychron / Nape 固有設定コマンド群
  KC_RGB                  = 168, // 0xA8: ライティング設定
  KC_HE                   = 169, // 0xA9: Hall Effect スイッチ設定
  KC_FACTORY              = 171, // 0xAB: 工場テスト / Chip ID 取得
  KC_SCREEN               = 172, // 0xAC: ディスプレイ / スクリーンの設定
}
```

---

## 3. レイヤー状態の取得 (`KC_GET_CURRENT_LAYER = 163 / 0xA3`)

完全読み取り専用（STRICT Read-Only）で、現在端末でアクティブになっているレイヤー情報を取得するための公式パケット構造です。

### 送信パケット (33バイト)
```text
[0] = 0x00  (Report ID)
[1] = 0xA3  (163: KC_GET_CURRENT_LAYER)
[2] = 0x00
[3] = 0xFF  (255)
[4..32] = 0x00
```

### 受信パケット構造 (32バイト)
```text
[0] = 0xA3  (163: コマンドヘッダ応答)
[1] = default_layer (デフォルトレイヤー番号: 0〜7)
[2] = overlay_layer (一時オーバーレイレイヤー番号。未適用時は 255 / 0xFF)
[3..31] = 予約 / 追加ステータス
```

### 判定アルゴリズム (公式 Keychron Launcher 準拠)
```typescript
function parseActiveLayer(responseBuffer: Uint8Array): number {
    const defaultLayer = responseBuffer[1];
    const overlayLayer = responseBuffer[2];
    
    if (overlayLayer !== 255) {
        return Math.max(defaultLayer, overlayLayer);
    }
    return defaultLayer;
}
```

---

## 4. Nape Pro 固有設定コマンド (`KC_MISC_CMD_GROUP = 167 / 0xA7`)

Nape Pro トラックボール固有機能（OctaShift 8方向角度設定やレイヤー切替）を制御するサブコマンド群の公式定数および送信・受信バイト仕様です。

### Nape サブコマンド enum 定数 (公式 JS ソースコードより抽出)
```typescript
enum KeychronNapeSubCommand {
  KC_USER_CMD_NAPE_GET_ORI               = 32, // 0x20: 全体 OctaShift 角度取得
  KC_USER_CMD_NAPE_GET_DPI               = 33, // 0x21: DPI 取得
  KC_USER_CMD_NAPE_SET_DPI               = 34, // 0x22: DPI 設定 (インデックス指定)
  KC_USER_CMD_NAPE_SET_DPI_VALUE         = 35, // 0x23: DPI 値指定 (インデックス + u16 LE)
  KC_USER_CMD_NAPE_GET_DPI_VALUE         = 36, // 0x24: DPI 値取得
  KC_USER_CMD_NAPE_SET_TAPHOLDS          = 37, // 0x25: TapHold 設定
  KC_USER_CMD_NAPE_GET_TAPHOLDS          = 38, // 0x26: TapHold 取得
  KC_USER_CMD_NAPE_SET_COMBOS            = 39, // 0x27: Combo 設定
  KC_USER_CMD_NAPE_GET_COMBOS            = 40, // 0x28: Combo 取得
  KC_USER_CMD_NAPE_SET_GESTURE           = 41, // 0x29: ジェスチャー設定
  KC_USER_CMD_NAPE_GET_GESTURE           = 42, // 0x2A: ジェスチャー取得
  KC_USER_CMD_NAPE_SET_PROFILE           = 43, // 0x2B: プロファイル設定
  KC_USER_CMD_NAPE_GET_PROFILE           = 44, // 0x2C: プロファイル取得
  KC_USER_CMD_NAPE_SET_LAYER             = 45, // 0x2D: アクティブレイヤー切り替え (※1-based index)
  KC_USER_CMD_NAPE_DEL_COMBOS            = 46, // 0x2E: Combo 削除
  KC_USER_CMD_NAPE_DEL_TAPHOLDS          = 47, // 0x2F: TapHold 削除
  KC_USER_CMD_NAPE_SET_FORCE_GESTURE_SCROLL = 50, // 0x32: ジェスチャー & スロールモード一括設定
  KC_USER_CMD_NAPE_GET_FORCE_GESTURE_SCROLL = 51, // 0x33: ジェスチャー & スロールモード一括取得
  KC_USER_CMD_NAPE_SET_ORI               = 52, // 0x34: 全体 OctaShift 角度設定
  KC_USER_CMD_NAPE_GET_CUSTOM_DPI        = 54, // 0x36: カスタム DPI 値取得
  KC_USER_CMD_NAPE_SET_CUSTOM_DPI        = 55, // 0x37: カスタム DPI 値設定
  KC_USER_CMD_NAPE_GET_LAYER_ORI         = 56, // 0x38: レイヤー別 OctaShift 角度取得
  KC_USER_CMD_NAPE_SET_LAYER_ORI         = 57, // 0x39: レイヤー別 OctaShift 角度設定
}
```

### パケット送信・受信詳細構造

| サブコマンド | コマンド送信構造 (Report ID 0x00 + 32B) | 応答構造 (Report 32B) | 備考 / 算出式 |
| :--- | :--- | :--- | :--- |
| **`KC_USER_CMD_NAPE_SET_FORCE_GESTURE_SCROLL`** | `[0x00, 167, 50, gesture, scroll]` | `[167, 50, status, ...]` | `gesture`: 1/0, `scroll`: 1/0 |
| **`KC_USER_CMD_NAPE_GET_FORCE_GESTURE_SCROLL`** | `[0x00, 167, 51]` | `[167, 51, gesture, scroll, ...]` | `gesture`: res[2], `scroll`: res[3] |
| **`KC_USER_CMD_NAPE_GET_DPI`** | `[0x00, 167, 33]` | `[167, 33, active_index, ...]` | `active_index`: res[2] (0〜4) |
| **`KC_USER_CMD_NAPE_SET_DPI`** | `[0x00, 167, 34, dpi_index]` | `[167, 34, ...]` | `dpi_index` (0〜4: 400/800/1800/3200/4000) |
| **`KC_USER_CMD_NAPE_SET_DPI_VALUE`** | `[0x00, 167, 35, dpi_index, dpi_lo, dpi_hi]` | `[167, 35, ...]` | 指定インデックスのDPI値変更 |
| **`KC_USER_CMD_NAPE_GET_DPI_VALUE`** | `[0x00, 167, 36, dpi_index]` | `[167, 36, dpi_lo, dpi_hi, ...]` | `dpi = res[2] \| (res[3] << 8)` |
| **`KC_USER_CMD_NAPE_SET_CUSTOM_DPI`** | `[0x00, 167, 55, dpi_lo, dpi_hi]` | `[167, 55, ...]` | `dpi` 16-bit LE (u16) |
| **`KC_USER_CMD_NAPE_GET_CUSTOM_DPI`** | `[0x00, 167, 54]` | `[167, 54, dpi_lo, dpi_hi, ...]` | `dpi = res[2] \| (res[3] << 8)` |
| **`KC_USER_CMD_NAPE_GET_LAYER_ORI`** | `[0x00, 167, 56, layer_id]` | `[167, 56, angle_div_45, ...]` | `角度 = 45 * res[2]` (※res[2]参照) |
| **`KC_USER_CMD_NAPE_SET_LAYER_ORI`** | `[0x00, 167, 57, layer_id, angle / 45]` | `[167, 57, ...]` | `angle / 45` (0〜7) |
| **`KC_USER_CMD_NAPE_GET_ORI`** | `[0x00, 167, 32]` | `[167, 32, angle_div_45, ...]` | `角度 = 45 * res[2]` |
| **`KC_USER_CMD_NAPE_SET_ORI`** | `[0x00, 167, 52, angle / 45]` | `[167, 52, ...]` | `angle / 45` (0〜7) |
| **`KC_USER_CMD_NAPE_SET_LAYER`** | `[0x00, 167, 45, 1_based_layer_id]`| `[167, 45, ...]` | `1..8` (1=Layer 0, 2=Layer 1...) |

---

## 5. キーマップ (EEPROM) 読み出し (`0x12` `DYNAMIC_KEYMAP_GET_BUFFER`)

標準 VIA Protocol コマンド `0x12` (`DYNAMIC_KEYMAP_GET_BUFFER`) を使用。

### Nape Pro のキーマップメモリ仕様
- **1レイヤーあたりのメモリサイズ**: **28 バイト (14 キーコード)**
- **キーコード長**: 2 バイト / キーコード (Big-Endian `u16`)
- **EEPROM バイト単位オフセット計算**:
  - `offset_in_bytes = 28 * layer_index`
  - (例: Layer 0 = 0, Layer 1 = 28, Layer 2 = 56, Layer 3 = 84, Layer 4 = 112, Layer 5 = 140, Layer 6 = 168, Layer 7 = 196)

---

## 6. 安全ガイドライン

> [!CAUTION]
> **誤送信による危険なリセットコマンドの禁止**
> - VIA コマンド `0x06` (`DYNAMIC_KEYMAP_RESET`) はデバイス設定を工場出荷にリセットするため、アクティブレイヤー取得などの読み取り目的で送信してはいけません。
> - アクティブレイヤー取得には必ず公式の **`0xA3` (163)** を使用してください。

---

## 7. 実装上の重要な作法と落とし穴ガイド (Implementation Pitfalls & Best Practices)

開発・デバッグ中にハマりやすい実機ファームウェア特有の挙動と解決策をまとめています。

### 7.1 レイヤー切り替えサブコマンド (`0xA7 45`) は 1-Based インデックス
- **落とし穴**: VIA Protocol の一般的なレイヤー番号は 0-based (`0..7`) ですが、Nape Pro のレイヤー切り替えサブコマンド `0xA7 45` (`KC_USER_CMD_NAPE_SET_LAYER`) は **1-based (`1..8`)** の数値（Layer 0 ＝ `1`, Layer 1 ＝ `2` ...）を要求します。
- **発生する現象**: `0` (Layer 0 のつもり) を送信すると、バックグラウンドの `0xA3` (現在アクティブレイヤー取得) で認識されたレイヤーと齟齬が生じ、前回のレイヤーへ一瞬で引き戻される（スナップバック）挙動が発生します。

### 7.2 OctaShift 設置角度による EEPROM インデックスマッピングの変動
- **落とし穴**: 設置角度（0° 縦置き / 90° 横置き等）によって、物理ボタンと `keycodes[0..13]` の割り当て参照インデックスが入れ替わります。
- **対策**: 単純なキーコード単体チェック（`keycodes[11] == 0x00D1` 等）だけで分岐させると 0° モード時に誤判定を起こすため、`keycodes[8] == 0x00D5 && keycodes[10] == 0x00D4` のように角度モード判定フラグを厳密に分離すること。

### 7.2.1 OctaShift 全8方向 (0°〜315°) 角度対応マトリクス

Keychron Launcher およびファームウェアにおける、全 8 方向の設置角度 (OctaShift) に対する EEPROM 配列インデックスの割り当て規則および UI 上のビジュアル配置マトリクスです。

| 設置角度 | 角度ステップ | 物理位置 (回転後の画角) | EEPROM 参照インデックス (有線 / 無線) | 代表的な標準/カスタムキー割り当て | UI ダイアグラム回転 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **0°** | `0 * 45°` | サイド左上 (03)<br>サイド右上 (04)<br>サイド左下 (01)<br>サイド右下 (02) | `keycodes[7]` / `[0]`<br>`keycodes[8]` / `[1]`<br>`keycodes[9]` / `[2]`<br>`keycodes[10]` / `[3]` | `Ball_Scroll` / `Cmd+Opt+←`<br>`Switch_8Dir` / `Cmd+Opt+→`<br>`Browser_Back` / `Cmd+Enter`<br>`Cycle_DPI` / `Cmd+Shift+Enter` | **0° (標準立位)** |
| **45°** | `1 * 45°` | 45° 時計回り回転 | `0°` の配列基準 + 45° 座標シフト | 0° 基準インデックス + 45° 回転描画 | **45° 右傾斜** |
| **90°** | `2 * 45°` | 回転時 左上 (01)<br>回転時 右上 (03)<br>回転時 右下 (04)<br>回転時 左下 (02) | `keycodes[8]` / `[1]`<br>`keycodes[10]` / `[3]`<br>`keycodes[11]` / `[4]`<br>`keycodes[12]` / `[5]` | `0x00D5` (英数)<br>`0x00D4` (かな)<br>`0x00D1` (進む)<br>`0x00D2` (戻る) | **90° 水平回転** |
| **135°** | `3 * 45°` | 135° 時計回り回転 | `90°` / `180°` 座標シフト | 135° 座標回転描画 | **135° 右下傾斜** |
| **180°** | `4 * 45°` | 180° 上下反転<br>回転時 左上 (02)<br>回転時 右上 (01)<br>回転時 左下 (04)<br>回転時 右下 (03) | `keycodes[10]` / `[3]`<br>`keycodes[9]` / `[2]`<br>`keycodes[8]` / `[1]`<br>`keycodes[7]` / `[0]` | 反転時位置に対応 | **180° 上下反転** |
| **225°** | `5 * 45°` | 225° 時計回り回転 | `180°` / `270°` 座標シフト | 225° 座標回転描画 | **225° 左下傾斜** |
| **270°** | `6 * 45°` | 270° 水平反転<br>回転時 左上 (04)<br>回転時 右上 (02)<br>回転時 左下 (03)<br>回転時 右下 (01) | `keycodes[11]` / `[4]`<br>`keycodes[12]` / `[5]`<br>`keycodes[10]` / `[3]`<br>`keycodes[8]` / `[1]` | 水平反転時位置に対応 | **270° 水平反転** |
| **315°** | `7 * 45°` | 315° 時計回り回転 | `270°` / `0°` 座標シフト | 315° 座標回転描画 | **315° 左傾斜** |

### 7.3 サブコマンド 56 (`KC_USER_CMD_NAPE_GET_LAYER_ORI`) の応答パケット構造
- **落とし穴**: 応答パケット `[0xA7, 56, ...]` において、`angle_div_45` (0〜7 : 0°〜315°) は **`buf[start_idx + 2]` (2番目のパラメータバイト)** に格納されています。
- **発生する現象**: `buf[start_idx + 3]`（ステータス/予約バイト）を参照すると常に `0` と評価され、定期ヘルスチェック時に全レイヤーの角度設定が 0° に上書きリセットされてしまいます。

### 7.4 ジョグダイヤル (スクロールリング) のデフォルト解釈
- **落とし穴**: 未割り当て（`0x0000`）のレイヤーにおけるジョグダイヤルの工場出荷時デフォルト機能は、スクロールではなく **Volume Up (`0x7E2B`) / Volume Down (`0x7E2C`)** です。
- **画面表示の順序**: 公式 Keychron Launcher に合わせ、上側スロットを **`↻ Volume Up` (時計回り)**、下側スロットを **`↺ Volume Down` (反時計回り)** として配置・描画します。

### 7.5 バックグラウンド HID ポーリングの衝突回避
- **落とし穴**: 全 8 レイヤーの EEPROM（28 バイト × 8 ＝ 224 バイト）を毎秒読み出し続けると、WebHID 経由で通信する公式 Keychron Launcher と HID バス上でパケット衝突を起こし、通信切断が発生します。
- **対策**: キーマップ配列の取得は接続時およびレイヤー切り替え時のみ行い、バックグラウンドポーリング（2.5秒周期）ではアクティブレイヤー (`0xA3`)・角度・DPI の最小限のステータス確認にとどめること。
