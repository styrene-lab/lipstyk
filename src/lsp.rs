use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use lipstyk::diagnostic::Severity;
use lipstyk::config::Config;
use lipstyk::Linter;

struct Backend {
    client: Client,
    linter: Arc<Linter>,
    open_files: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    fn lint_and_publish(&self, uri: Url, text: &str) -> Vec<Diagnostic> {
        let filename = uri.path();
        let score = match self.linter.lint_source(filename, text) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        score
            .diagnostics
            .iter()
            .map(|d| {
                let severity = match d.severity {
                    Severity::Hint => Some(DiagnosticSeverity::HINT),
                    Severity::Warning => Some(DiagnosticSeverity::WARNING),
                    Severity::Slop => Some(DiagnosticSeverity::ERROR),
                };

                let line = d.line.saturating_sub(1) as u32;

                Diagnostic {
                    range: Range {
                        start: Position { line, character: 0 },
                        end: Position { line, character: 999 },
                    },
                    severity,
                    code: Some(NumberOrString::String(d.rule.to_string())),
                    code_description: None,
                    source: Some("lipstyk".to_string()),
                    message: d.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "lipstyk LSP initialized")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        self.open_files.write().await.insert(uri.clone(), text.clone());
        let diagnostics = self.lint_and_publish(uri.clone(), &text);
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.open_files.write().await.insert(uri.clone(), text.clone());
            let diagnostics = self.lint_and_publish(uri.clone(), &text);
            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(text) = self.open_files.read().await.get(&uri) {
            let diagnostics = self.lint_and_publish(uri.clone(), text);
            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.open_files.write().await.remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let config = Config::discover(std::path::Path::new("."));
    let linter = Linter::with_defaults()
        .exclude_tests(true)
        .with_config(config);

    let (service, socket) = LspService::new(|client| Backend {
        client,
        linter: Arc::new(linter),
        open_files: Arc::new(RwLock::new(HashMap::new())),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
