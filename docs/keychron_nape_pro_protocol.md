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
  KC_USER_CMD_NAPE_SET_LAYER             = 45, // 0x2D: アクティブレイヤー切り替え
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
| **`KC_USER_CMD_NAPE_GET_LAYER_ORI`** | `[0x00, 167, 56, layer_id]` | `[167, 56, angle_div_45, ...]` | `角度 = 45 * res[2]` |
| **`KC_USER_CMD_NAPE_SET_LAYER_ORI`** | `[0x00, 167, 57, layer_id, angle / 45]` | `[167, 57, ...]` | `angle / 45` (0〜7) |
| **`KC_USER_CMD_NAPE_GET_ORI`** | `[0x00, 167, 32]` | `[167, 32, angle_div_45, ...]` | `角度 = 45 * res[2]` |
| **`KC_USER_CMD_NAPE_SET_ORI`** | `[0x00, 167, 52, angle / 45]` | `[167, 52, ...]` | `angle / 45` (0〜7) |
| **`KC_USER_CMD_NAPE_SET_LAYER`** | `[0x00, 167, 45, layer_id]` | `[167, 45, ...]` | アクティブレイヤー切り替え |

### 4.1 アクティブ DPI 正確取得シーケンス
Keychron Nape Pro で現在アクティブな DPI 値（4000 DPI 等）を正確に取得するには、サブコマンド 54（カスタムDPI指定値）ではなく、以下の 2 ステップ手順を使用します：
1. **`0xA7 33` (`KC_USER_CMD_NAPE_GET_DPI`)**: アクティブな DPI レベルインデックス (`active_index`: 0〜4) を取得。
2. **`0xA7 36` (`KC_USER_CMD_NAPE_GET_DPI_VALUE`)**: パケット `[0x00, 167, 36, active_index]` を送信し、そのレベルに設定されている正確な DPI 値 (`res[2] | (res[3] << 8)`) を読み出し。

### 4.2 設定データの記憶スコープ構造 (データ保持単位)

| 設定項目 | スコープ | 通信コマンド | 説明 |
| :--- | :--- | :--- | :--- |
| **キーマップ (ボタン割り当て)** | **レイヤー単位** (Layer 0〜7) | `0x12` (`DYNAMIC_KEYMAP`) | 8つのレイヤーごとに独立したキーコード配列を保持 |
| **OctaShift 設置角度** | **レイヤー単位** (Layer 0〜7) | `167 56 / 57` | レイヤーごとに独立したオフセット角度 (0〜315°) を保持 |
| **ポインター感度 (DPI)** | **デバイス共通** (グローバル) | `167 33/34/35/55` | レイヤーに依らずデバイス全体で共通 |
| **スクロール / ジェスチャーモード** | **デバイス共通** (グローバル) | `167 50 / 51` | レイヤーに依らずデバイス全体で共通 |
| **ポーリングレート** | **デバイス共通** (グローバル) | `167 14` / `0x0C` | レイヤーに依らずデバイス全体で共通 |

---

## 5. キーマップ (EEPROM) 読み出し (`0x12` `DYNAMIC_KEYMAP_GET_BUFFER`)

標準 VIA Protocol コマンド `0x12` (`DYNAMIC_KEYMAP_GET_BUFFER`) を使用。

### Nape Pro のキーマップメモリ仕様
- **1レイヤーあたりのメモリサイズ**: **28 バイト (14 キーコード)**
- **キーコード長**: 2 バイト / キーコード (Big-Endian `u16`)
- **EEPROM バイト単位オフセット計算 (公式 Keychron Launcher 準拠)**:
  - VIA プロトコル `0x12` (`DYNAMIC_KEYMAP_GET_BUFFER`) の `offset` フィールドは **16-bit バイトオフセット** です。
  - `offset_in_bytes = 28 * layer_index`
  - (例: Layer 0 = 0, Layer 1 = 28, Layer 2 = 56, Layer 3 = 84, Layer 4 = 112, Layer 5 = 140, Layer 6 = 168, Layer 7 = 196)

#### 送信パケット構造 (Layer $L$ 読み出し)
```text
[0] = 0x00  (Report ID)
[1] = 0x12  (DYNAMIC_KEYMAP_GET_BUFFER)
[2] = (28 * L) >> 8    (offset_hi: バイト単位のオフセット上位バイト)
[3] = (28 * L) & 0xFF  (offset_lo: バイト単位のオフセット下位バイト)
[4] = 28               (count: 取得するバイト数 = 28バイト = 14キーコード)
```

#### 受信パケット構造 (32バイト)
```text
[0] = 0x12  (ヘッダ)
[1..2] = offset
[3] = length (14)
[4..31] = 14個の Big-Endian u16 キーコード配列 (Zt[i+1] | (Zt[i] << 8))
```

### 5.1 キーコード配列から物理ボタンへの固定インデックスマッピング規則 (Direct Index Mapping)

1レイヤーの 14 キーコード配列 (`keycodes[0..13]`) は、前半 7 キー `keycodes[0..6]` (無線モード) および後半 7 キー `keycodes[7..13]` (USB 有線モード / オーバーライド) で構成されています。
Keychron Launcher の逆コンパイルおよび動作検証に基づく各ボタンの絶対参照インデックス構造は以下の通りです：

| ボタン名 | ボタンID | 物理位置 | 配列インデックスと参照先 (有線 / 無線) |
| :--- | :--- | :--- | :--- |
| **03 (G3)** | `5` | サイド左上 | `keycodes[7]` / `keycodes[0]` (標準: `0x522B` ボールスクロール) |
| **04 (G4)** | `6` | サイド右上 | `keycodes[8]` / `keycodes[1]` (標準: `0x522A` 8方向切替) |
| **01 (G1)** | `3` | サイド左下 | `keycodes[9]` / `keycodes[2]` (標準: `0x00D2` 戻る) |
| **02 (G2)** | `4` | サイド右下 | `keycodes[10]` / `keycodes[3]` (標準: `0x7E2D` DPI切替) |
| **M1** | `1` | 左メインボタン | `keycodes[11]` / `keycodes[4]` (標準: `0x0001` / `0x7E29` 左クリック) |
| **M2** | `2` | 右メインボタン | `keycodes[12]` / `keycodes[5]` (標準: `0x0002` / `0x7E29` 右クリック) |
| **スクロール (上)** | `7` | リング上回転 | `keycodes[13]` / `keycodes[6]` (標準: `0x7E2B` Vol_Up) |
| **スクロール (下)** | `8` | リング下回転 | `0x7E2C` (標準: `0x7E2C` Vol_Down) |

### 5.2 Keychron Nape Pro デフォルトレイヤー構成マトリクス

EEPROM バッファが未割り当て（`0x0000`）の際、Keychron Launcher 上で表示・適用されるレイヤー別標準デフォルト仕様です（Layer 0〜7 すべてで共通の構成となります）。

| ボタン名 | ID | 物理位置 | 標準デフォルト仕様 (Layer 0〜7 共通) |
| :--- | :--- | :--- | :--- |
| **M1** | `1` | 左メインボタン | `Click_Left` (左クリック / `0x0001` or `0x7E29`) |
| **M2** | `2` | 右メインボタン | `Click_Right` (右クリック / `0x0002` or `0x7E29`) |
| **01 (G1)** | `3` | サイド左下 | `Browser_Back` (戻る / `0x00D2`) |
| **02 (G2)** | `4` | サイド右下 | `Cycle_DPI` (DPI切替 / `0x7E2D`) |
| **03 (G3)** | `5` | サイド左上 | `Ball_Scroll` (ボールスクロール / `0x522B`) |
| **04 (G4)** | `6` | サイド右上 | `Switch_8Dir` (8方向切り替え / `0x522A`) |
| **スクロール (上)** | `7` | リング上回転 | `Vol_Up` (Volume Up / `0x7E2B`) |
| **スクロール (下)** | `8` | リング下回転 | `Vol_Down` (Volume Down / `0x7E2C`) |

---

## 6. 安全ガイドライン

> [!CAUTION]
> **誤送信による危険なリセットコマンドの禁止**
> - VIA コマンド `0x06` (`DYNAMIC_KEYMAP_RESET`) はデバイス設定を工場出荷にリセットするため、アクティブレイヤー取得などの読み取り目的で送信してはいけません。
> - アクティブレイヤー取得には必ず公式の **`0xA3` (163)** を使用してください。
