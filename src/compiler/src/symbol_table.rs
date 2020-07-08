use crate::node::{GrammarClassVarType, GrammarItemType};
use std::collections::HashMap;

type SymbolName = String;

#[derive(Debug)]
enum ClassVarKind {
    Field,
    Static,
}

impl From<&GrammarClassVarType> for ClassVarKind {
    fn from(other: &GrammarClassVarType) -> Self {
        match other {
            GrammarClassVarType::Static => Self::Static,
            GrammarClassVarType::Field => Self::Field,
        }
    }
}

#[derive(Debug)]
struct EntryClass {
    typ: String,
    kind: ClassVarKind,
    index: usize,
}

impl From<&EntryClass> for Entry {
    fn from(other: &EntryClass) -> Self {
        Self {
            index: other.index,
            typ: other.typ.clone(),
            kind: match other.kind {
                ClassVarKind::Field => "field".to_string(),
                ClassVarKind::Static => "static".to_string(),
            },
        }
    }
}

#[derive(Debug)]
pub enum SubVarKind {
    Argument,
    Var,
}

#[derive(Debug)]
struct EntrySub {
    typ: String,
    kind: SubVarKind,
    index: usize,
}

impl From<&EntrySub> for Entry {
    fn from(other: &EntrySub) -> Self {
        Self {
            index: other.index,
            typ: other.typ.clone(),
            kind: match other.kind {
                SubVarKind::Argument => "argument".to_string(),
                SubVarKind::Var => "var".to_string(),
            },
        }
    }
}

type DictClass = HashMap<SymbolName, EntryClass>;
type DictSub = HashMap<SymbolName, EntrySub>;

#[derive(Debug, Default)]
pub struct SymbolTable {
    class: DictClass,
    sub: DictSub,
}

// Generic entry
#[derive(Debug, Default, Clone)]
pub struct Entry {
    pub typ: String,
    pub kind: String,
    pub index: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lookup(&self, name: &SymbolName) -> Option<Entry> {
        if let Some(e) = self.sub.get(name) {
            return Some(e.into());
        }
        if let Some(e) = self.class.get(name) {
            return Some(e.into());
        }
        None
    }

    pub fn define_class_var(
        &mut self,
        name: &SymbolName,
        kind: &GrammarClassVarType,
        typ: &GrammarItemType,
    ) {
        let dict = &mut self.class;
        let index = dict.len();
        let entry = EntryClass {
            typ: type_as_string(typ),
            kind: kind.into(),
            index,
        };
        dict.insert(name.clone(), entry);
    }

    pub fn define_subroutine_var(
        &mut self,
        name: &SymbolName,
        kind: SubVarKind,
        typ: &GrammarItemType,
    ) {
        let dict = &mut self.sub;
        let index = dict.len();
        let entry = EntrySub {
            typ: type_as_string(typ),
            kind: kind.into(),
            index,
        };
        dict.insert(name.clone(), entry);
    }

    pub fn reset_subroutine_table(&mut self) {
        self.sub = Default::default();
    }
}

pub fn type_as_string(typ: &GrammarItemType) -> String {
    match typ {
        GrammarItemType::Int => "int",
        GrammarItemType::Char => "char",
        GrammarItemType::Boolean => "boolean",
        GrammarItemType::Class(ident) => ident.as_str(),
    }
    .to_string()
}
