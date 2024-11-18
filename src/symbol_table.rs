use std::collections::HashMap;

use oxc_index::IndexVec;

use crate::span::Span;

oxc_index::define_index_type! {
    pub struct SymbolId = u32;
    IMPL_RAW_CONVERSIONS = true;
}

oxc_index::define_index_type! {
    pub struct ReferenceId = u32;
    IMPL_RAW_CONVERSIONS = true;
}
pub type SymbolIdToSpan = IndexVec<SymbolId, Span>;

pub type ReferenceIdToReference = IndexVec<ReferenceId, Reference>;

#[derive(Default, Debug)]
pub struct SymbolTable {
    pub span_to_symbol_id: HashMap<Span, SymbolId>,
    pub symbol_id_to_span: SymbolIdToSpan,
    pub reference_id_to_reference: ReferenceIdToReference,
    pub span_to_reference_id: HashMap<Span, ReferenceId>,
    pub symbol_id_to_references: HashMap<SymbolId, Vec<ReferenceId>>,
}

#[derive(Debug)]
pub struct Reference {
    pub span: Span,
    pub symbol_id: Option<SymbolId>,
}

impl SymbolTable {
    pub fn add_symbol(&mut self, span: Span) -> SymbolId {
        let symbol_id = self.symbol_id_to_span.push(span.clone());
        self.span_to_symbol_id.insert(span.clone(), symbol_id);
        symbol_id
    }

    pub fn add_reference(&mut self, span: Span, symbol_id: Option<SymbolId>) {
        let reference_id = self.reference_id_to_reference.push(Reference {
            span: span.clone(),
            symbol_id,
        });
        self.span_to_reference_id.insert(span, reference_id);
        if let Some(symbol_id) = symbol_id {
            self.symbol_id_to_references
                .entry(symbol_id)
                .or_default()
                .push(reference_id);
        }
    }
}
