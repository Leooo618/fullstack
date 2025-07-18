use common::{CreateMessage, Message};
use leptos::{prelude::*, reactive::spawn_local};

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! { <Index /> }
}

#[component]
fn Index() -> impl IntoView {
    let input_message = RwSignal::new(String::new());

    let get_messages = LocalResource::new(move || get_messages());

    let error = RwSignal::new(String::new());
    let has_error = move || error.get().len() > 0;

    view! {
        <Show when=move || has_error()>
            <Alert message=error.get().to_string() />
        </Show>

        <div class="flex flex-col items-center gap-4 pt-12">
            <h1 class="text-3xl font-bold">"Header 1"</h1>
            <span class="w-1/2 h-px bg-gray-300"></span>
            <div class="flex justify-center space-x-2">
                <input
                    type="text"
                    class="mt-0.5 w-96 rounded border-gray-300 shadow-sm sm:text-sm"
                    id="message"
                    bind:value=input_message
                />

                <button
                    class="bg-blue-500 text-white px-4 py-2 rounded"
                    on:click=move |_| {
                        spawn_local(async move {
                            match create_message(&input_message.get()).await {
                                Ok(_) => {
                                    input_message.set(String::new());
                                    get_messages.refetch();
                                }
                                Err(e) => {
                                    error.set(e.to_string());
                                }
                            }
                        });
                    }
                >
                    "发送"
                </button>
            </div>

            {move || Suspend::new(async move {
                let result = get_messages.await;
                match result {
                    Ok(messages) => view! { <DisplayMessage messages=messages /> }.into_any(),
                    Err(e) => {
                        error.set(e.to_string());
                        view! {}.into_any()
                    }
                }
            })}

        </div>
    }
}

#[component]
fn DisplayMessage(messages: Vec<Message>) -> impl IntoView {
    view! {
        <ol class="list-decimal space-y-2">
            {messages.into_iter().map(|item| view! { <li>{item.content}</li> }).collect_view()}
        </ol>
    }
}

#[component]
fn Alert(message: String) -> impl IntoView {
    view! {
        <div role="alert" class="rounded-md border border-gray-300 bg-white p-4 shadow-sm">
            <div class="flex items-start gap-4">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="1.5"
                    stroke="currentColor"
                    class="size-6 text-green-600"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>

                <div class="flex-1">
                    <strong class="font-medium text-gray-900">Error ...</strong>

                    <p class="mt-0.5 text-sm text-gray-700">{message}</p>
                </div>

                <button
                    class="-m-3 rounded-full p-1.5 text-gray-500 transition-colors hover:bg-gray-50 hover:text-gray-700"
                    type="button"
                    aria-label="Dismiss alert"
                >
                    <span class="sr-only">Dismiss popup</span>

                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>
        </div>
    }
}

async fn create_message(message: &str) -> Result<(), ApiError> {
    let resp = gloo_net::http::Request::put("http://127.0.0.1:8080/message")
        .json(&CreateMessage {
            content: message.to_string(),
        })?
        .send()
        .await?;
    check_response_status(&resp).await?;
    Ok(())
}

async fn get_messages() -> Result<Vec<Message>, ApiError> {
    let resp = gloo_net::http::Request::get("http://127.0.0.1:8080/message")
        .send()
        .await?;
    check_response_status(&resp).await?;
    let messages = resp.json::<Vec<Message>>().await?;
    Ok(messages)
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<gloo_net::Error> for ApiError {
    fn from(_: gloo_net::Error) -> Self {
        Self {
            message: "系统内部错误".into(),
        }
    }
}

async fn check_response_status(resp: &gloo_net::http::Response) -> Result<(), ApiError> {
    if resp.status() >= 300 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ApiError { message });
    }
    Ok(())
}
