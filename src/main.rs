use dashmap::DashMap;
use l_lang::{
    AstNode, CompileResult, Formatter, SymbolId, SymbolKind, Type, compile, find_node_at_offset,
};
use log::debug;
use ropey::Rope;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
    semanticast_map: DashMap<String, CompileResult>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,

            capabilities: ServerCapabilities {
                document_formatting_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
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
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("nrs".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: vec![
                                        SemanticTokenType::FUNCTION,
                                        SemanticTokenType::VARIABLE,
                                        SemanticTokenType::PARAMETER,
                                        SemanticTokenType::STRUCT,
                                        SemanticTokenType::PROPERTY,
                                    ],
                                    token_modifiers: vec![],
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentChange {
            uri: params.text_document.uri.to_string(),
            text: &params.text_document.text,
        })
        .await;
        debug!("file opened!");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentChange {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri.to_string(),
        })
        .await;
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        debug!("file saved!");
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        debug!("file closed!");
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let definition = self.get_definition(params);
        Ok(definition)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let references = self.get_references(uri, position, params.context.include_declaration);
        Ok(references)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        let semantic_tokens = self.build_semantic_tokens(&uri);
        if let Some(tokens) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })));
        }
        Ok(None)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri.to_string();
        let range = params.range;
        let semantic_tokens = self.build_semantic_tokens_range(&uri, range);
        Ok(semantic_tokens.map(|data| {
            SemanticTokensRangeResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })
        }))
    }

    async fn inlay_hint(
        &self,
        params: tower_lsp::lsp_types::InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        Ok(self.build_inlay_hints(params.text_document.uri.as_ref()))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = self.get_completion(params);
        Ok(completions.map(CompletionResponse::Array))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let workspace_edit = self.get_rename_edit(uri, position, new_name);
        Ok(workspace_edit)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(self.format_text(params))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed!");
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        debug!("workspace folders changed!");
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        debug!("watched files have changed!");
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("command executed!");

        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        semanticast_map: DashMap::new(),
        document_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

impl Backend {
    fn format_text(&self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let uri = params.text_document.uri.to_string();
        let rope = self.document_map.get(&uri)?;
        let semantic_result = self.semanticast_map.get(&uri)?;
        let formatter = Formatter::new(80);
        let formatted_text = formatter.format(semantic_result.program.file(), &rope.to_string());
        Some(vec![TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(
                    rope.len_lines() as u32,
                    rope.line(rope.len_lines() - 1).len_chars() as u32,
                ),
            },
            new_text: formatted_text,
        }])
    }

    fn build_inlay_hints(&self, uri: &str) -> Option<Vec<InlayHint>> {
        let semantic_result = self.semanticast_map.get(uri)?;
        let rope = self.document_map.get(uri)?;
        let bindings = &semantic_result.semantic.bindings;
        let hints = bindings
            .iter_enumerated()
            .filter_map(|(symbol_id, type_info)| {
                if semantic_result.semantic.get_symbol_kind(symbol_id)
                    != l_lang::SymbolKind::Variable
                {
                    return None;
                }
                let span = semantic_result.semantic.get_symbol_span(symbol_id);
                let end = offset_to_position(span.end as usize, &rope)?;
                let inly_hint_parts = match type_info.ty {
                    Type::Struct(id) => {
                        let mut parts = vec![];
                        parts.push(InlayHintLabelPart {
                            value: ": ".to_string(),
                            ..Default::default()
                        });
                        let span = semantic_result.semantic.get_symbol_span(id);
                        let start = offset_to_position(span.start as usize, &rope)?;
                        let end = offset_to_position(span.end as usize, &rope)?;
                        let location = Location::new(
                            Url::parse(uri)
                                .unwrap_or_else(|_| Url::from_directory_path(uri).unwrap()),
                            Range::new(start, end),
                        );
                        parts.push(InlayHintLabelPart {
                            value: type_info.ty.format_literal_type(&semantic_result.semantic),
                            location: Some(location),
                            ..Default::default()
                        });
                        InlayHintLabel::LabelParts(parts)
                    }
                    _ => InlayHintLabel::String(format!(
                        ": {}",
                        type_info.ty.format_literal_type(&semantic_result.semantic)
                    )),
                };
                Some(InlayHint {
                    position: Position::new(end.line, end.character),
                    label: inly_hint_parts,
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                })
            })
            .collect::<Vec<_>>();

        Some(hints)
    }

    fn get_definition(&self, params: GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let rope = self.document_map.get(&uri)?;

        let compilation_result = self.semanticast_map.get(&uri)?;
        let offset = position_to_offset(position, &rope)?;
        if let Some(interval) = compilation_result
            .semantic
            .span_to_symbol
            .find(offset, offset + 1)
            .next()
        {
            let start = offset_to_position(interval.start, &rope)?;
            let end = offset_to_position(interval.stop, &rope)?;
            let location = Location::new(
                params.text_document_position_params.text_document.uri,
                Range::new(start, end),
            );
            return Some(GotoDefinitionResponse::Scalar(location));
        };
        let ref_id = compilation_result
            .semantic
            .span_to_reference
            .find(offset, offset + 1)
            .next()?
            .val;
        let symbol_id = compilation_result.semantic.references[ref_id]?;
        let symbol_span = compilation_result.semantic.get_symbol_span(symbol_id);
        let start = offset_to_position(symbol_span.start as usize, &rope)?;
        let end = offset_to_position(symbol_span.end as usize, &rope)?;
        let location = Location::new(
            params.text_document_position_params.text_document.uri,
            Range::new(start, end),
        );

        Some(GotoDefinitionResponse::Scalar(location))
    }

    fn get_references(
        &self,
        uri: String,
        position: Position,
        include_self: bool,
    ) -> Option<Vec<Location>> {
        let rope = self.document_map.get(&uri)?;
        let compilation_result = self.semanticast_map.get(&uri)?;
        let offset = position_to_offset(position, &rope)?;
        let symbol_id = compilation_result.semantic.get_symbol_at(offset)?;

        let mut references = Vec::new();
        let uri = Url::parse(&uri).unwrap_or_else(|_| Url::from_directory_path(&uri).unwrap());
        if include_self {
            // Include the symbol definition itself
            let symbol_span = compilation_result.semantic.get_symbol_span(symbol_id);
            let start = offset_to_position(symbol_span.start as usize, &rope)?;
            let end = offset_to_position(symbol_span.end as usize, &rope)?;
            references.push(Location::new(uri.clone(), Range::new(start, end)));
        }
        // Find the reference at the current position
        let ref_ids = compilation_result.semantic.get_symbol_references(symbol_id);

        references.extend(ref_ids.iter().filter_map(|ref_id| {
            let span = compilation_result.semantic.reference_spans[*ref_id];
            let start = offset_to_position(span.start as usize, &rope)?;
            let end = offset_to_position(span.end as usize, &rope)?;
            Some(Location::new(uri.clone(), Range::new(start, end)))
        }));
        Some(references)
    }

    fn get_rename_edit(
        &self,
        uri: String,
        position: Position,
        new_name: String,
    ) -> Option<WorkspaceEdit> {
        let all_reference = self.get_references(uri.clone(), position, true)?;

        let edits = all_reference
            .into_iter()
            .map(|item| TextEdit {
                range: item.range,
                new_text: new_name.clone(),
            })
            .collect::<Vec<_>>();

        // Create workspace edit with the text edits
        let parsed_uri =
            Url::parse(&uri).unwrap_or_else(|_| Url::from_directory_path(&uri).unwrap());
        let mut edit_map = std::collections::HashMap::new();
        edit_map.insert(parsed_uri, edits);

        Some(WorkspaceEdit::new(edit_map))
    }

    fn get_struct_id_from_field(
        &self,
        field_expr: &l_lang::ExprField,
        semantic_result: &CompileResult,
    ) -> Option<SymbolId> {
        let mut access_arr = vec![];
        let mut cur = field_expr.object.as_ref()?;
        loop {
            match cur.as_ref() {
                l_lang::Expr::Field(field_expr) => {
                    access_arr.push(field_expr.field.as_ref()?.name.clone());
                    cur = field_expr.object.as_ref()?;
                }
                l_lang::Expr::Name(_name_expr) => {
                    break;
                }
                _ => {
                    return None;
                }
            }
        }
        access_arr.reverse();

        let reference_id = semantic_result
            .semantic
            .get_reference_at(field_expr.object.as_ref()?.span().start as usize)?;
        let symbol_id = semantic_result.semantic.references[reference_id]?;
        let ty_info = semantic_result.semantic.get_symbol_type(symbol_id)?;
        let Type::Struct(mut struct_id) = ty_info.ty else {
            return None;
        };
        
        for field_name in access_arr {
            let struct_def = semantic_result.semantic.structs.get(&struct_id)?;
            let field = struct_def.fields.iter().find(|f| f.name == field_name)?;
            let Type::Struct(next_struct_id) = field.ty else {
                return None;
            };
            struct_id = next_struct_id;
        }
        Some(struct_id)

    }

    fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let text_doc_position = params.text_document_position;
        let uri = text_doc_position.text_document.uri.to_string();
        let semantic_result = self.semanticast_map.get(&uri)?;
        let rope = self.document_map.get(&uri)?;
        let offset = position_to_offset(text_doc_position.position, &rope)?;

        let mut items = Vec::new();

        // Try to find the AST node at the current position
        if let Some(nearest_node) =
            find_node_at_offset(semantic_result.program.file(), offset as u32)
        {
            match nearest_node {
                // Field access completion: suggest available fields/members
                AstNode::ExprField(field_expr) => {
                    let struct_id = self.get_struct_id_from_field(field_expr, &semantic_result)?;
                    let struct_def = semantic_result.semantic.structs.get(&struct_id)?;
                    struct_def.fields.iter().for_each(|field| {
                        items.push(CompletionItem {
                            label: field.name.clone(),
                            kind: Some(CompletionItemKind::FIELD),
                            detail: Some(format!(
                                ": {}",
                                field.ty.format_literal_type(&semantic_result.semantic)
                            )),
                            insert_text: Some(field.name.clone()),
                            ..Default::default()
                        });
                    });
                }
                _ => {
                    // Default: suggest all available symbols
                    let bindings = &semantic_result.semantic.bindings;
                    bindings
                        .iter_enumerated()
                        .for_each(|(symbol_id, type_info)| {
                            let symbol_kind = semantic_result.semantic.get_symbol_kind(symbol_id);
                            let span = semantic_result.semantic.get_symbol_span(symbol_id);

                            let name_slice =
                                rope.byte_slice(span.start as usize..span.end as usize);
                            if let Ok(name) = std::str::from_utf8(
                                name_slice.bytes().collect::<Vec<_>>().as_slice(),
                            ) {
                                let (kind, detail) = match symbol_kind {
                                    l_lang::SymbolKind::Variable => (
                                        Some(CompletionItemKind::VARIABLE),
                                        Some(format!(
                                            ": {}",
                                            type_info
                                                .ty
                                                .format_literal_type(&semantic_result.semantic)
                                        )),
                                    ),
                                    l_lang::SymbolKind::Function => {
                                        (Some(CompletionItemKind::FUNCTION), None)
                                    }
                                    l_lang::SymbolKind::Struct => {
                                        (Some(CompletionItemKind::STRUCT), None)
                                    }
                                    _ => (None, None),
                                };

                                items.push(CompletionItem {
                                    label: name.to_string(),
                                    kind,
                                    detail,
                                    insert_text: Some(name.to_string()),
                                    ..Default::default()
                                });
                            }
                        });
                }
            }
        } else {
            // No node found, suggest all available symbols
            let bindings = &semantic_result.semantic.bindings;
            bindings
                .iter_enumerated()
                .for_each(|(symbol_id, type_info)| {
                    let symbol_kind = semantic_result.semantic.get_symbol_kind(symbol_id);
                    let span = semantic_result.semantic.get_symbol_span(symbol_id);

                    let name_slice = rope.byte_slice(span.start as usize..span.end as usize);
                    if let Ok(name) =
                        std::str::from_utf8(name_slice.bytes().collect::<Vec<_>>().as_slice())
                    {
                        let (kind, detail) = match symbol_kind {
                            l_lang::SymbolKind::Variable => (
                                Some(CompletionItemKind::VARIABLE),
                                Some(format!(
                                    ": {}",
                                    type_info.ty.format_literal_type(&semantic_result.semantic)
                                )),
                            ),
                            l_lang::SymbolKind::Function => {
                                (Some(CompletionItemKind::FUNCTION), None)
                            }
                            l_lang::SymbolKind::Struct => {
                                (Some(CompletionItemKind::STRUCT), None)
                            }
                            _ => (None, None),
                        };

                        items.push(CompletionItem {
                            label: name.to_string(),
                            kind,
                            detail,
                            insert_text: Some(name.to_string()),
                            ..Default::default()
                        });
                    }
                });
        }
        Some(items)
    }

    async fn on_change(&self, item: TextDocumentChange<'_>) {
        let rope = Rope::from_str(item.text);
        let compile_result = compile(item.text);
        let mut diagnostics = compile_result
            .diagnostics
            .iter()
            .flat_map(|d| {
                d.labels.iter().filter_map(|label| {
                    let start = offset_to_position(label.range.start, &rope)?;
                    let end = offset_to_position(label.range.end, &rope)?;
                    let diag = Diagnostic {
                        range: Range::new(start, end),
                        severity: None,
                        code: None,
                        code_description: None,
                        source: None,
                        message: format!("{:?}", d.message),
                        related_information: None,
                        tags: None,
                        data: None,
                    };
                    Some(diag)
                })
            })
            .collect::<Vec<_>>();
        compile_result.semantic.errors.iter().for_each(|sem_err| {
            let span = sem_err.span;
            let start = offset_to_position(span.start as usize, &rope);
            let end = offset_to_position(span.end as usize, &rope);
            if let (Some(start), Some(end)) = (start, end) {
                let diag = Diagnostic {
                    range: Range::new(start, end),
                    severity: None,
                    code: None,
                    code_description: None,
                    source: None,
                    message: sem_err.message.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diag);
            }
        });

        let uri =
            Url::parse(&item.uri).unwrap_or_else(|_| Url::from_directory_path(&item.uri).unwrap());
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
        self.semanticast_map
            .insert(item.uri.clone(), compile_result);
        self.document_map.insert(item.uri.clone(), rope);
    }

    fn build_semantic_tokens(&self, uri: &str) -> Option<Vec<SemanticToken>> {
        let semantic_result = self.semanticast_map.get(uri)?;
        let rope = self.document_map.get(uri)?;

        // Collect all tokens from symbols and references
        // Token type indices correspond to LEGEND_TYPE order:
        // 0: FUNCTION, 1: VARIABLE, 2: PARAMETER, 3: STRUCT, 4: PROPERTY (field)
        let mut incomplete_tokens: Vec<(usize, usize, u32)> = Vec::new(); // (start, length, token_type)

        // Add symbol definitions
        for (symbol_id, span) in semantic_result.semantic.symbol_spans.iter_enumerated() {
            let kind = semantic_result.semantic.get_symbol_kind(symbol_id);
            let token_type = match kind {
                SymbolKind::Function => 0,   // FUNCTION
                SymbolKind::Variable => 1,   // VARIABLE
                SymbolKind::Parameter => 2,  // PARAMETER
                SymbolKind::Struct => 3,     // STRUCT
                SymbolKind::Field => 4,      // PROPERTY
            };
            incomplete_tokens.push((span.start as usize, (span.end - span.start) as usize, token_type));
        }

        // Add references (they reference symbols, so use the symbol's kind)
        for (ref_id, span) in semantic_result.semantic.reference_spans.iter_enumerated() {
            if let Some(symbol_id) = semantic_result.semantic.references[ref_id] {
                let kind = semantic_result.semantic.get_symbol_kind(symbol_id);
                let token_type = match kind {
                    SymbolKind::Function => 0,   // FUNCTION
                    SymbolKind::Variable => 1,   // VARIABLE
                    SymbolKind::Parameter => 2,  // PARAMETER
                    SymbolKind::Struct => 3,     // STRUCT
                    SymbolKind::Field => 4,      // PROPERTY
                };
                incomplete_tokens.push((span.start as usize, (span.end - span.start) as usize, token_type));
            }
        }

        // Sort by start position
        incomplete_tokens.sort_by(|a, b| a.0.cmp(&b.0));

        // Convert to LSP SemanticToken format with delta encoding
        let mut pre_line: u32 = 0;
        let mut pre_start: u32 = 0;

        let semantic_tokens = incomplete_tokens
            .iter()
            .filter_map(|(start, length, token_type)| {
                // Convert byte offset to line and character
                let line = rope.try_byte_to_line(*start).ok()? as u32;
                let line_start_byte = rope.try_line_to_byte(line as usize).ok()?;
                let char_offset = *start - line_start_byte;

                let delta_line = line - pre_line;
                let delta_start = if delta_line == 0 {
                    char_offset as u32 - pre_start
                } else {
                    char_offset as u32
                };

                let token = SemanticToken {
                    delta_line,
                    delta_start,
                    length: *length as u32,
                    token_type: *token_type,
                    token_modifiers_bitset: 0,
                };

                pre_line = line;
                pre_start = char_offset as u32;

                Some(token)
            })
            .collect::<Vec<_>>();

        Some(semantic_tokens)
    }

    fn build_semantic_tokens_range(&self, uri: &str, range: Range) -> Option<Vec<SemanticToken>> {
        let semantic_result = self.semanticast_map.get(uri)?;
        let rope = self.document_map.get(uri)?;

        // Convert range to byte offsets
        let start_offset = position_to_offset(range.start, &rope)?;
        let end_offset = position_to_offset(range.end, &rope)?;

        // Collect all tokens from symbols and references within the range
        let mut incomplete_tokens: Vec<(usize, usize, u32)> = Vec::new();

        // Add symbol definitions within range
        for (symbol_id, span) in semantic_result.semantic.symbol_spans.iter_enumerated() {
            let token_start = span.start as usize;
            if token_start >= start_offset && token_start < end_offset {
                let kind = semantic_result.semantic.get_symbol_kind(symbol_id);
                let token_type = match kind {
                    SymbolKind::Function => 0,
                    SymbolKind::Variable => 1,
                    SymbolKind::Parameter => 2,
                    SymbolKind::Struct => 3,
                    SymbolKind::Field => 4,
                };
                incomplete_tokens.push((token_start, (span.end - span.start) as usize, token_type));
            }
        }

        // Add references within range
        for (ref_id, span) in semantic_result.semantic.reference_spans.iter_enumerated() {
            let token_start = span.start as usize;
            if token_start >= start_offset && token_start < end_offset {
                if let Some(symbol_id) = semantic_result.semantic.references[ref_id] {
                    let kind = semantic_result.semantic.get_symbol_kind(symbol_id);
                    let token_type = match kind {
                        SymbolKind::Function => 0,
                        SymbolKind::Variable => 1,
                        SymbolKind::Parameter => 2,
                        SymbolKind::Struct => 3,
                        SymbolKind::Field => 4,
                    };
                    incomplete_tokens.push((token_start, (span.end - span.start) as usize, token_type));
                }
            }
        }

        // Sort by start position
        incomplete_tokens.sort_by(|a, b| a.0.cmp(&b.0));

        // Convert to LSP SemanticToken format with delta encoding
        let mut pre_line: u32 = 0;
        let mut pre_start: u32 = 0;

        let semantic_tokens = incomplete_tokens
            .iter()
            .filter_map(|(start, length, token_type)| {
                let line = rope.try_byte_to_line(*start).ok()? as u32;
                let line_start_byte = rope.try_line_to_byte(line as usize).ok()?;
                let char_offset = *start - line_start_byte;

                let delta_line = line - pre_line;
                let delta_start = if delta_line == 0 {
                    char_offset as u32 - pre_start
                } else {
                    char_offset as u32
                };

                let token = SemanticToken {
                    delta_line,
                    delta_start,
                    length: *length as u32,
                    token_type: *token_type,
                    token_modifiers_bitset: 0,
                };

                pre_line = line;
                pre_start = char_offset as u32;

                Some(token)
            })
            .collect::<Vec<_>>();

        Some(semantic_tokens)
    }
}

struct TextDocumentChange<'a> {
    uri: String,
    text: &'a str,
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

fn position_to_offset(position: Position, rope: &Rope) -> Option<usize> {
    let line_char_offset = rope.try_line_to_char(position.line as usize).ok()?;
    let slice = rope.slice(0..line_char_offset + position.character as usize);
    Some(slice.len_bytes())
}
