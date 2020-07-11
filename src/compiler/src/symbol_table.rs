use crate::node::{GrammarClassVarType, GrammarItemType};
use std::collections::HashMap;

type SymbolName = String;

#[derive(Debug, Eq, PartialEq, Hash)]
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
    index: u16,
}

impl From<&EntryClass> for Entry {
    fn from(other: &EntryClass) -> Self {
        Self {
            index: other.index,
            typ: other.typ.clone(),
            kind: match other.kind {
                ClassVarKind::Field => "this".to_string(),
                ClassVarKind::Static => "static".to_string(),
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum SubVarKind {
    Argument,
    Var,
}

#[derive(Debug)]
struct EntrySub {
    typ: String,
    kind: SubVarKind,
    index: u16,
}

impl From<&EntrySub> for Entry {
    fn from(other: &EntrySub) -> Self {
        Self {
            index: other.index,
            typ: other.typ.clone(),
            kind: match other.kind {
                SubVarKind::Argument => "argument".to_string(),
                SubVarKind::Var => "local".to_string(),
            },
        }
    }
}

#[derive(Debug, Default)]
struct DictWithIndex<TEntryMap, TIndexMap> {
    entry_dict: TEntryMap,
    index_dict: TIndexMap,
}

type DictClass = DictWithIndex<HashMap<SymbolName, EntryClass>, HashMap<ClassVarKind, u16>>;
type DictSub = DictWithIndex<HashMap<SymbolName, EntrySub>, HashMap<SubVarKind, u16>>;

#[derive(Debug, Default)]
pub struct SymbolTable {
    class: DictClass,
    sub: DictSub,
}

// Generic entry
#[derive(Debug, Default, Clone)]
pub struct Entry {
    pub typ: String,
    // What kind of memory segment
    pub kind: String,
    pub index: u16,
}

impl SymbolTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lookup(&self, name: &SymbolName) -> Option<Entry> {
        if let Some(e) = self.sub.entry_dict.get(name) {
            return Some(e.into());
        }
        if let Some(e) = self.class.entry_dict.get(name) {
            return Some(e.into());
        }
        None
    }

    pub fn count_instance_fields(&self) -> u16 {
        *self
            .class
            .index_dict
            .get(&ClassVarKind::Field)
            .unwrap_or(&0)
    }

    pub fn define_class_var(
        &mut self,
        name: &SymbolName,
        kind: &GrammarClassVarType,
        typ: &GrammarItemType,
    ) {
        let dict = &mut self.class;
        let index = dict.index_dict.entry(kind.into()).or_insert(0);
        let entry = EntryClass {
            typ: type_as_string(typ),
            kind: kind.into(),
            index: index.clone(),
        };
        *index += 1;
        dict.entry_dict.insert(name.clone(), entry);
    }

    pub fn define_subroutine_var(
        &mut self,
        name: &SymbolName,
        kind: SubVarKind,
        typ: &GrammarItemType,
    ) {
        let dict = &mut self.sub;
        let index = dict.index_dict.entry(kind.clone().into()).or_insert(0);
        let entry = EntrySub {
            typ: type_as_string(typ),
            kind: kind.into(),
            index: index.clone(),
        };
        *index += 1;
        dict.entry_dict.insert(name.clone(), entry);
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
