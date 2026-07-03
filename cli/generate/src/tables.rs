use std::collections::BTreeMap;

use pyo3::prelude::pyclass;

use super::{
    nfa::CharacterSet,
    rules::{Alias, Symbol, TokenSet},
};
pub type ProductionInfoId = usize;
pub type ParseStateId = usize;
pub type LexStateId = usize;

use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use rustc_hash::FxHasher;

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseAction {
    Accept(),
    Shift {
        state: ParseStateId,
        is_repetition: bool,
    },
    ShiftExtra(),
    Recover(),
    Reduce {
        symbol: Symbol,
        child_count: usize,
        dynamic_precedence: i32,
        production_id: ProductionInfoId,
    },
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GotoAction {
    Goto(ParseStateId),
    ShiftExtra(),
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParseTableEntry {
    #[pyo3(get)]
    pub actions: Vec<ParseAction>,
    #[pyo3(get)]
    pub reusable: bool,
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParseState {
    #[pyo3(get)]
    pub id: ParseStateId,
    #[pyo3(get)]
    pub reserved_words: TokenSet,
    #[pyo3(get)]
    pub lex_state_id: usize,
    #[pyo3(get)]
    pub anti_lex_state_id: Option<usize>,
    #[pyo3(get)]
    pub external_lex_state_id: usize,
    #[pyo3(get)]
    pub core_id: usize,

    pub terminal_entries: IndexMap<Symbol, ParseTableEntry, BuildHasherDefault<FxHasher>>,
    pub nonterminal_entries: IndexMap<Symbol, GotoAction, BuildHasherDefault<FxHasher>>,
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FieldLocation {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub inherited: bool,
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProductionInfo {
    #[pyo3(get)]
    pub alias_sequence: Vec<Option<Alias>>,
    #[pyo3(get)]
    pub field_map: BTreeMap<String, Vec<FieldLocation>>,
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParseTable {
    #[pyo3(get)]
    pub states: Vec<ParseState>,
    #[pyo3(get)]
    pub symbols: Vec<Symbol>,
    #[pyo3(get)]
    pub production_infos: Vec<ProductionInfo>,
    #[pyo3(get)]
    pub max_aliased_production_length: usize,
    #[pyo3(get)]
    pub external_lex_states: Vec<TokenSet>,
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AdvanceAction {
    #[pyo3(get)]
    pub state: LexStateId,
    #[pyo3(get)]
    pub in_main_token: bool,
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexState {
    #[pyo3(get)]
    pub accept_action: Option<Symbol>,
    #[pyo3(get)]
    pub eof_action: Option<AdvanceAction>,
    #[pyo3(get)]
    pub advance_actions: Vec<(CharacterSet, AdvanceAction)>,
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LexTable {
    #[pyo3(get)]
    pub states: Vec<LexState>,
}

impl ParseTableEntry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reusable: true,
            actions: Vec::new(),
        }
    }
}

impl ParseState {
    pub fn is_end_of_non_terminal_extra(&self) -> bool {
        self.terminal_entries
            .contains_key(&Symbol::end_of_nonterminal_extra())
    }

    pub fn referenced_states(&self) -> impl Iterator<Item = ParseStateId> + '_ {
        self.terminal_entries
            .iter()
            .flat_map(|(_, entry)| {
                entry.actions.iter().filter_map(|action| match action {
                    ParseAction::Shift { state, .. } => Some(*state),
                    _ => None,
                })
            })
            .chain(self.nonterminal_entries.iter().filter_map(|(_, action)| {
                if let GotoAction::Goto(state) = action {
                    Some(*state)
                } else {
                    None
                }
            }))
    }

    pub fn update_referenced_states<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &Self) -> usize,
    {
        let mut updates = Vec::new();
        for (symbol, entry) in &self.terminal_entries {
            for (i, action) in entry.actions.iter().enumerate() {
                if let ParseAction::Shift { state, .. } = action {
                    let result = f(*state, self);
                    if result != *state {
                        updates.push((*symbol, i, result));
                    }
                }
            }
        }
        for (symbol, action) in &self.nonterminal_entries {
            if let GotoAction::Goto(other_state) = action {
                let result = f(*other_state, self);
                if result != *other_state {
                    updates.push((*symbol, 0, result));
                }
            }
        }
        for (symbol, action_index, new_state) in updates {
            if symbol.is_non_terminal() {
                self.nonterminal_entries
                    .insert(symbol, GotoAction::Goto(new_state));
            } else {
                let entry = self.terminal_entries.get_mut(&symbol).unwrap();
                if let ParseAction::Shift { is_repetition, .. } = entry.actions[action_index] {
                    entry.actions[action_index] = ParseAction::Shift {
                        state: new_state,
                        is_repetition,
                    };
                }
            }
        }
    }
}
