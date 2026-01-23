use crate::ast::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DebugInfo {
    // Maps instruction pointer (index in instructions vector) to source span
    pub source_map: HashMap<usize, Span>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self { source_map: HashMap::new() }
    }

    pub fn insert(&mut self, ip: usize, span: Span) {
        self.source_map.insert(ip, span);
    }

    pub fn get(&self, ip: usize) -> Option<&Span> {
        self.source_map.get(&ip)
    }
}
