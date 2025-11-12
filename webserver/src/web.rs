use alloc::{format, string::String};
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use log::info;
use picoserve::{
    AppBuilder, AppRouter, Config, Router, Server, Timeouts, make_static, response::IntoResponse,
    routing,
};

pub const WEB_TASK_POOL_SIZE: usize = 2;
const INDEX_HTML: &str = include_str!("index.html");
const PRE_PLACEHOLDER: &str = "{{PRE_CONTENT}}";
const ESP32_ASCII: &str = include_str!("esp32_logo.txt");

pub struct Application;

impl AppBuilder for Application {
    type PathRouter = impl routing::PathRouter;

    fn build_app(self) -> Router<Self::PathRouter> {
        Router::new()
            .route("/", routing::get(index_page))
            .route("/stats", routing::get(stats_page))
    }
}

pub struct WebApp {
    pub router: &'static Router<<Application as AppBuilder>::PathRouter>,
    pub config: &'static Config<Duration>,
}

impl Default for WebApp {
    fn default() -> Self {
        let router = make_static!(AppRouter<Application>, Application.build_app());
        let config = make_static!(
            Config<Duration>,
            Config::new(Timeouts {
                start_read_request: Some(Duration::from_secs(5)),
                persistent_start_read_request: Some(Duration::from_secs(5)),
                read_request: Some(Duration::from_secs(1)),
                write: Some(Duration::from_secs(1)),
            })
        );

        Self { router, config }
    }
}

impl WebApp {
    pub fn spawn_tasks(&self, spawner: &Spawner, stack: Stack<'static>) {
        for id in 0..WEB_TASK_POOL_SIZE {
            spawner.must_spawn(web_task(id, stack, self.router, self.config));
        }
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: Stack<'static>,
    router: &'static AppRouter<Application>,
    config: &'static Config<Duration>,
) {
    let port = 8001;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    let server = Server::new(router, &config, &mut http_buffer);

    info!("Starting server {id} on port {port}");
    server
        .listen_and_serve(id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await;
}

async fn index_page() -> impl IntoResponse {
    html_response(render_with_pre(ESP32_ASCII))
}

async fn stats_page() -> impl IntoResponse {
    let stats = heap_info();
    html_response(render_with_pre(&stats))
}

fn heap_info() -> String {
    format!("{}", esp_alloc::HEAP.stats()).replace("| Used", "|\nUsed")
}

fn render_with_pre(pre_content: &str) -> String {
    INDEX_HTML.replace(PRE_PLACEHOLDER, pre_content)
}

fn html_response(body: String) -> impl IntoResponse {
    (("Content-Type", "text/html; charset=utf-8"), body)
}
