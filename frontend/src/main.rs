use common::{CreateMessage, Message};
use dioxus::prelude::*;
use gloo_net::http::Request;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Index {}
    }
}

#[component]
fn Index() -> Element {
    let mut input_message = use_signal(|| String::new());

    let mut messages = use_resource(|| async { get_messages().await });

    rsx! {

        div { class: "flex flex-col items-center gap-4 pt-12",
            h1 { class: "text-3xl font-bold", "# Header 1" }
            span { class: "w-1/2 h-px bg-gray-300" }
            div { class: "flex justify-center space-x-2",
                input {
                    class: "mt-0.5 w-96 rounded border-gray-300 shadow-sm sm:text-sm",
                    id: "message",
                    placeholder: "请输入消息",
                    value: input_message,
                    oninput: move |e| input_message.set(e.value()),
                }
                button {
                    class: "bg-blue-500 text-white px-4 py-2 rounded",
                    onclick: move |_| async move {
                        match create_message(&input_message()).await {
                            Ok(_) => {
                                input_message.set(String::new());
                                messages.restart();
                            }
                            Err(e) => {
                                input_message.set(e.message)
                            }
                        }
                    },
                    "发送"
                }
            }

            if let Some(Ok(items)) = &*messages.read() {
                ol { class: "list-decimal space-y-2",
                    for item in items {
                        li { "{item.content}" }
                    }
                }
            }
        }
    }
}

async fn get_messages() -> Result<Vec<Message>, ApiError> {
    let response = Request::get("http://127.0.0.1:8080/message").send().await?;
    check_response_status(&response).await?;
    let messages = response.json::<Vec<Message>>().await?;
    return Ok(messages);
}

async fn create_message(message: &str) -> Result<(), ApiError> {
    let resp = Request::put("http://127.0.0.1:8080/message")
        .json(&CreateMessage {
            content: message.to_string(),
        })?
        .send()
        .await?;
    check_response_status(&resp).await?;

    Ok(())
}

async fn check_response_status(resp: &gloo_net::http::Response) -> Result<(), ApiError> {
    if resp.status() >= 300 {
        let body = resp.text().await?;
        return Err(ApiError { message: body });
    }
    Ok(())
}

#[derive(Debug)]
struct ApiError {
    pub message: String,
}

impl From<gloo_net::Error> for ApiError {
    fn from(value: gloo_net::Error) -> Self {
        ApiError {
            message: format!("系统内部错误: {value}"),
        }
    }
}
