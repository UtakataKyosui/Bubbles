# XQ (ActivityPub Protocol Buffer版) 調査報告書

## 概要
XQは、Misskeyプロジェクト（misskey-dev）によって提案されている、マイクロブログ向けの新しい高効率連合プロトコルです。ActivityPubがJSON（JSON-LD）を使用するのに対し、XQはProtocol Buffers（Protobuf）を使用することで、通信の効率化と型安全性、開発の容易さを目指しています。

## 調査結果

### 1. プロジェクトの現状
- **ステータス**: WIP（Work In Progress）。
- **リポジトリ**: [misskey-dev/xq](https://github.com/misskey-dev/xq)
- **採用状況**: 現時点ではMisskey本家での採用予定はなく、研究・実験的な段階です。

### 2. 技術的特徴
- **Protocol Buffersの採用**: 
  - バイナリ形式によるメッセージサイズの削減。
  - スキーマ定義（`.proto`）による型安全性の確保。
  - 多言語でのコード生成が可能。
- **効率化**: 
  - マイクロブログ用途に特化し、不要なデータやボイラープレートを排除。
  - 複数のメッセージを1つのリクエストにまとめるバッチ処理のサポート。

### 3. プロトコル定義 (現状)
現在の `protos/activity.proto` は非常にミニマルで、概念実証段階と思われます。

```protobuf
edition = "2023";

package xq;

message Post {
  string uri = 1;
}

message Activity {
  string uri = 1;
}
```

実用的なSNSプロトコルとして機能させるには、以下のようなフィールドの定義が必要になると考えられます：
- **Note/Post**: 内容 (content)、投稿者 (actor)、公開日時 (published)、公開範囲 (to/cc) など。
- **Actor/User**: ユーザー名、表示名、アイコン、公開鍵など。
- **Activity**: Create, Update, Delete, Follow, Like などのアクションタイプ。

## Bubblesへの適用について (Issue 2 & 8)

### Issue 2: 分散プロトコル選定
- **評価**: XQは現時点では「標準」としては確立されておらず、エコシステムも存在しません。
- **位置づけ**: Bubbles独自の実験的機能、あるいは将来的な差別化要因としての採用が適しています。メインの連合プロトコルとしてはActivityPubを採用しつつ、Bubbles同士（あるいはXQ対応サーバー間）の通信最適化オプションとして実装するのが現実的です。

### Issue 8: XQを実現する
- **実装アプローチ**:
  1.  **プロトコル定義の拡張**: ActivityPubの語彙（Note, Person等）を参考に、Bubblesに必要なデータを網羅した `.proto` ファイルを設計・作成する。
  2.  **通信基盤の構築**: gRPCまたはHTTP/2上でのProtobuf通信の実装。
  3.  **ハイブリッド対応**: 外部とはActivityPub (JSON) で通信し、内部や対応サーバー間ではXQ (Protobuf) で通信するアーキテクチャの検討。

## 次のステップ提案
Issue 8「XQを実現する」を進めるための第一歩として、Bubbles用のプロトタイプ的な `.proto` 定義を作成することを提案します。

```protobuf
syntax = "proto3";
package bubbles.xq;

message TextNote {
  string id = 1;
  string content = 2;
  string author_id = 3;
  int64 created_at = 4; // Unix timestamp
}

message CreateActivity {
  string id = 1;
  string actor_id = 2;
  TextNote note = 3;
}
```

## プロトタイプ実装 (apps/xq-prototype)
Rust (`prost` + `axum`) を用いたプロトタイプ実装を行いました。
これにより、Protocol Buffersで定義したデータ構造をHTTP Bodyとして送受信できることを確認しました。

### 実装構成
- **Crate**: `apps/xq-prototype`
- **Protocol**: `proto/bubbles_xq.proto` (上記定義を使用)
- **Server**: `src/main.rs` (Axumサーバー, `/api/activity` エンドポイント)
- **Client**: `examples/client.rs` (検証用クライアント)

### 動作確認結果
クライアントからProtobufエンコードされたActivityを送信し、サーバー側でデコード・ログ出力・レスポンス返却が正常に行われることを確認しました。

```log
// Server Log
INFO xq_prototype: Server listening on 127.0.0.1:3000
INFO xq_prototype: Received Activity: ID=activity-789
INFO xq_prototype: Actor: user-456
INFO xq_prototype: Note Content: Hello, XQ World! This is a test message via Protobuf.
```

## ベンチマーク結果
ActivityPub (JSON) と XQ (Protobuf) の処理速度およびデータサイズを比較しました。
※ Rustの `criterion` (速度) および データサイズ計算による結果。

### 1. データサイズ比較
同じ内容の投稿データ（本文約140文字程度）での比較。

| 形式 | サイズ (Bytes) | 備考 |
| :--- | :--- | :--- |
| **ActivityPub (JSON)** | 414 bytes | `@context` やプロパティ名を含む |
| **XQ (Protobuf)** | **210 bytes** | **約 49% 削減** |

### 2. 処理速度比較
シリアライズ（構造体→バイナリ/文字列）およびデシリアライズ（バイナリ/文字列→構造体）の速度。

| 処理 | JSON (ns) | Protobuf (ns) | 倍率 |
| :--- | :--- | :--- | :--- |
| **Serialization** | ~538 ns | **~65 ns** | **約 8.3倍 高速** |
| **Deserialization** | ~835 ns | **~245 ns** | **約 3.4倍 高速** |

### 考察
Protobufを採用することで、通信量は約半分に削減され、データ処理速度も数倍から8倍程度高速化されることが確認できました。特にモバイル環境や通信帯域が限られた環境、大量のメッセージを処理するサーバー間通信において、XQの優位性は非常に高いと言えます。


