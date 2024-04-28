# mui-misskey-websocket
Misskeyの[StreamAPI](https://misskey-hub.net/ja/docs/for-developers/api/streaming/)にWebSocketで接続し、指定ユーザーの画像投稿があればDiscordのwebhookで送信する  
[shuttle](https://www.shuttle.rs/)で動作させる

# 仕様したcrateのドキュメント
- [websocket](https://docs.rs/websocket/latest/websocket/all.html)  
[websocket::ClientBuilder](https://docs.rs/websocket/latest/websocket/client/builder/struct.ClientBuilder.html)  
[ClientBuilder.connect](https://docs.rs/websocket/latest/websocket/client/builder/struct.ClientBuilder.html#method.connect)  
[websocket::Client](https://docs.rs/websocket/latest/websocket/client/sync/struct.Client.html)  
[websocket::stream::sync::NetworkStream](https://docs.rs/websocket/latest/websocket/stream/sync/trait.NetworkStream.html)  
[websocket::sender::Writer](https://docs.rs/websocket/latest/websocket/sender/struct.Writer.html)  

- [shuttle_runtime](https://docs.rs/shuttle-runtime/latest/shuttle_runtime/)  
shuttle_scretsはshuttle_runtimeに統合されている

# `Secrets.toml`の構成
```
# MisskeyWebsocketの環境変数
MISSKEY_HOST="misskey.io"
MISSKEY_TOKEN="aaaaaaaaaaaaaaaaaa"
TARGET_USER_ID="aaaaaaaaa"
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/hogehoge"
```

# shuttleの操作
```
# 手元で実行
$ cargo shuttle run

# gitでコミットせずにデプロイ
$ cargo shuttle deploy --allow-dirty

# デプロイ
$ cargo shuttle deploy

```