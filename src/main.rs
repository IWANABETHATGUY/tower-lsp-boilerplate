use chumsky::Parser;
use diagnostic_ls::chumsky::parse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let rope = ropey::Rope::from_str(&params.content_changes[0].text);
        let result = {
            let res = parse(&params.content_changes[0].text);
            res.1
        };
        let diagnostics = result
            .into_iter()
            .filter_map(|item| {
                let msg = format!(
                    "{}{}, expected {}",
                    if item.found().is_some() {
                        "Unexpected token"
                    } else {
                        "Unexpected end of input"
                    },
                    if let Some(label) = item.label() {
                        format!(" while parsing {}", label)
                    } else {
                        String::new()
                    },
                    if item.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        item.expected()
                            .map(|expected| match expected {
                                Some(expected) => expected.to_string(),
                                None => "end of input".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                );
                let span = item.span();
                let diagnostic = || -> ropey::Result<Diagnostic> {
                    let start_line = rope.try_char_to_line(span.start)?;
                    let first_char = rope.try_line_to_char(start_line)?;
                    let start_column = span.start - first_char;

                    let end_line = rope.try_char_to_line(span.end)?;
                    let first_char = rope.try_line_to_char(end_line)?;
                    let end_column = span.end - first_char;
                    Ok(Diagnostic::new_simple(
                        Range::new(
                            Position::new(start_line as u32, start_column as u32),
                            Position::new(end_line as u32, end_column as u32),
                        ),
                        msg,
                    ))
                }();
                diagnostic.ok()
            })
            .collect::<Vec<_>>();
        if !diagnostics.is_empty() {
            self.client
                .publish_diagnostics(
                    params.text_document.uri.clone(),
                    diagnostics,
                    Some(params.text_document.version),
                )
                .await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .send_notification::<CustomNotification>(TestParams {
                a: "test".to_string(),
            })
            .await;
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }
}
#[derive(Debug, Deserialize, Serialize)]
struct TestParams {
    a: String,
}

enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = TestParams;
    const METHOD: &'static str = "custom/notification";
}
impl Backend {
    async fn test(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        Ok(params)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client })
        .method("custom/request", Backend::test)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
