use alicia::prelude::*;

//================================================================

use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

//================================================================

#[derive(Debug, Default)]
struct State {
    scope: Option<Scope>,
}

impl State {
    fn file_begin(&mut self, path: String, file: String) -> std::result::Result<(), Error> {
        self.scope = Some(Builder::default().with_data(path, file)?.build_scope()?);
        Ok(())
    }

    fn file_close(&mut self) {
        self.scope = None;
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    state: Mutex<State>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "Alicia Language Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, parameter: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("server initialized!: {parameter:#?}"),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, parameter: DidOpenTextDocumentParams) {
        {
            let mut state = self.state.lock().unwrap();
            let _ = state.file_begin(
                parameter.text_document.uri.path().to_string(),
                parameter.text_document.text.clone(),
            );
        }

        self.client
            .log_message(MessageType::INFO, format!("did open!: {parameter:#?}"))
            .await;
    }

    async fn did_close(&self, parameter: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("did close!: {parameter:#?}"))
            .await;
    }

    async fn hover(&self, parameter: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, format!("hover!: {parameter:#?}"))
            .await;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("hello from hover".to_string())),
            range: None,
        }))
    }
}

fn foo() {}

pub async fn server_main() {
    foo();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        state: State::default().into(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
