<img width="1920" height="1080" alt="Bubbles" src="https://github.com/user-attachments/assets/743d1d13-84db-408f-8895-6601dc7b357a" />

# Bubbles is What?

BubblesはNostrを採用したTUIで起動できるSNSクライアントです

---

## なぜ Bubbles を作ったか

### SNS自鯖の夢と現実

もともと自分のSNSを持ちたいという動機から、Misskeyサーバを自前で運用していた。
しかし2つの問題にぶつかった。

**コスト問題**
MisskeyはNode.js + PostgreSQL + Redis の構成で、ActivityPub連合によって他インスタンスからのトラフィックも流れ込む。
個人運用には重すぎるインフラコストがかかっていた。
GoでMisskeyバックエンドを書き直してリソースを削減しようとしたが、現実的なスコープではなかった。

**法律問題**
日本でSNSを運営すると、プロバイダ責任制限法の対象となる。
2022年の改正により、違法投稿への対応義務がさらに強化された。
個人開発者が法的対応を継続するのは困難だった。

### NoStr という答え

NoStrプロトコルでは、サーバ（Relay）はメッセージを中継するだけの役割を担う。
コンテンツの責任はクライアント側にあり、Relayは「通信を提供しているだけ」という立場が取りやすい。
ActivityPubと比べてプロトコルが単純（WebSocket + JSON）なため、Rustで軽量に実装できる。

これらの問題を解決する設計として、BubblesはNoStr + Rust + TUIの構成を選択した。

### 過去のリポジトリとの関係

| リポジトリ | 役割 | 結末 |
|-----------|------|------|
| MyMisskey | Misskeyサーバ自鯖運用 | コスト・法律問題で終了 |
| MisskeyBackendRefactoringGoLang | GoでMisskey書き直し | スコープが大きすぎて停止 |
| UtakataSNSFactory | SNS開発の構想リポジトリ | Bubblesに統合 |
| DreamBoards | 夢SNSプロジェクト | Bubblesに統合 |

Bubblesはこれらの試行錯誤の末に辿り着いた、現時点での答えである。
