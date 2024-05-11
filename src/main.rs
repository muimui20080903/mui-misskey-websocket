use serde_json::{json, Map, Value};
use websocket::{
    client::sync::Client, stream::sync::NetworkStream, ClientBuilder, Message, OwnedMessage,
};

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<MyService, shuttle_runtime::Error> {
    // チャンネルへの接続ごとのid
    //  適当な文字列
    let id: String = String::from("awf2nawo0w8a3");
    // MisskeyのストリーミングAPI(homeTimeLine)に接続
    let mut client: Client<Box<dyn NetworkStream + Send>> =
        connect_to_misskey_streaming_api(&id, &secrets).await;

    println!("Connected to Misskey Streaming API");

    // メッセージの受信
    // ループしてメッセージを受信
    loop {
        let message_result: Result<OwnedMessage, websocket::WebSocketError> = client.recv_message();
        match message_result {
            // メッセージ受信成功
            Ok(message) => {
                // メッセージ内容を変数に格納
                let message_text: String = match message {
                    OwnedMessage::Text(text) => text,
                    _ => continue,
                };
                // メッセージ内容をオブジェクトに変換
                let message_json: Value = serde_json::from_str(&message_text).unwrap();
                let message_object: &Map<String, Value> = message_json.as_object().unwrap();
                if message_object["body"]["id"] != id {
                    continue;
                }

                // メッセージが対象のノートであれば、Discordにメッセージを送信
                if is_target_note(message_object, &secrets) {
                    println!("Received target note");
                    let note_info: String = generate_note_info(message_object);
                    send_discord_message(note_info, &secrets).await.unwrap();
                }

                // ディレイ
                std::thread::sleep(std::time::Duration::from_secs(60));
            }

            // データがない場合
            Err(websocket::result::WebSocketError::NoDataAvailable) => {
                // println!("No data available, retrying...");
                // 再接続処理
                client = connect_to_misskey_streaming_api(&id, &secrets).await;
                // ディレイ
                std::thread::sleep(std::time::Duration::from_secs(60));
                continue;
            }

            // その他のエラー
            Err(e) => {
                // メッセージ受信エラー
                println!("Message receive error: {:?}", e);
                break;
            }
        };
    }

    Ok(MyService {})
}

// Customize this struct with things from `shuttle_main` needed in `bind`,
// such as secrets or database connections
struct MyService {}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for MyService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        // Start your service and bind to the socket address
        Ok(())
    }
}

// misskeyのストリーミングAPIに接続
async fn connect_to_misskey_streaming_api(
    id: &str,
    secrets: &shuttle_runtime::SecretStore,
) -> Client<Box<dyn NetworkStream + Send>> {
    // パラメータの読み込み
    let host: String = secrets
        .get("MISSKEY_HOST")
        .expect("MISSKEY_HOST must be set");
    let token: String = secrets
        .get("MISSKEY_TOKEN")
        .expect("MISSKEY_TOKEN must be set");

    // StreamingAPIのURLの生成
    let url: String = format!(
        "wss://{host}/streaming?i={token}",
        host = host,
        token = token
    );

    // ストリームに接続
    let mut client: Client<Box<dyn NetworkStream + Send>> = ClientBuilder::new(&url)
        .unwrap()
        .connect(None)
        .expect("Failed to connect to Misskey Streaming API");

    // println!("Connected to Misskey Streaming API");

    // チャンネル(homeTimeLine)に接続
    let message = generate_message_to_connect_hometimeline_ch(id);
    client
        .send_message(&message)
        .expect("Failed to connect to channel");

    // メッセージを受信するため接続したクライアントを返す
    client
}

// チャンネルに接続するために送信するメッセージを生成
fn generate_message_to_connect_hometimeline_ch(id: &str) -> Message<'_> {
    // Create the message
    let message_json: Value = json!({
        "type": "connect",
        "body": {
            "channel": "homeTimeline",
            "id": id,
        }
    });

    // Convert the message to a string
    Message::text(message_json.to_string())
}

// メッセージが対象のノートであるかどうかを判定
fn is_target_note(
    message_object: &serde_json::Map<String, Value>,
    secrets: &shuttle_runtime::SecretStore,
) -> bool {
    // bodyを取得
    let message_object = message_object["body"].as_object().unwrap();

    // メッセージがノートでない場合はfalseを返す
    if message_object["type"].as_str().unwrap() != "note" {
        return false;
    }

    // ファイルが添付されていない場合はfalseを返す
    if message_object["body"]["files"]
        .as_array()
        .unwrap()
        .is_empty()
    {
        return false;
    }

    // メッセージが指定ユーザーのものでない場合はfalseを返す
    let target_user_id: String = secrets
        .get("TARGET_USER_ID")
        .expect("TARGET_USER_ID must be set"); // やんよさんのID
    if message_object["body"]["userId"].as_str().unwrap() != target_user_id {
        return false;
    }

    true
}

// Discordに送信するメッセージを生成
fn generate_note_info(message_object: &serde_json::Map<String, Value>) -> String {
    // メッセージの情報を取得

    // ノートのurl
    let note_id: String = message_object["body"]["body"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let url: String = format!(
        "[note](https://misskey.io/notes/{note_id})",
        note_id = note_id
    );

    // 添付ファイルのurl
    let files: String = message_object["body"]["body"]["files"]
        .as_array() // 配列として受け取る
        .unwrap()
        .iter()
        .enumerate() // インデックスを取得
        // インデックスとファイル情報を結合してMarkdownの文字列に変換
        .map(|(index, file)| {
            let img_url = file["url"].as_str().unwrap();
            let img_name = format!(
                "[{index}枚目]({img_url})",
                index = index + 1,
                img_url = img_url
            );
            img_name
        })
        // 改行で結合
        .collect::<Vec<String>>()
        .join("\n");

    // メッセージの情報を結合して戻り値として返す
    format!("{uri}\n{files}", uri = url, files = files)
}

// Discordにメッセージを送信
async fn send_discord_message(
    message: String,
    secrets: &shuttle_runtime::SecretStore,
) -> Result<(), Box<dyn std::error::Error>> {
    // Discordへのメッセージ送信
    let discord_webhook_url: String = secrets
        .get("DISCORD_WEBHOOK_URL")
        .expect("DISCORD_WEBHOOK_URL must be set");

    let client = reqwest::Client::new();
    let res = client
        .post(discord_webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json!({ "content": message }).to_string())
        .send()
        .await
        .expect("Failed to send message to Discord");

    // POSTリクエストの結果を確認
    if res.status().is_success() {
        println!("Send message to Discord:\n{}", message);
    } else {
        println!("Failed to send message to Discord:\n{:?}", res);
    }

    Ok(())
}
