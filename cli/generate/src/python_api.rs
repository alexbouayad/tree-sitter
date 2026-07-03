use pyo3::prelude::pymodule;

#[pymodule]
mod ts_generate {
    use std::collections::{HashMap, HashSet};
    use std::fs;

    use pyo3::prelude::{pyclass, pymethods};

    use crate::build_tables::build_tables;
    use crate::node_types::get_variable_info;
    use crate::parse_grammar::parse_grammar;
    use crate::prepare_grammar::prepare_grammar;
    use crate::rename_variables;

    #[pymodule_export]
    use crate::build_tables::Tables;
    #[pymodule_export]
    use crate::grammars::{
        ExternalToken, LexicalGrammar, LexicalVariable, PrecedenceEntry, Production,
        ProductionStep, ReservedWordSetId, SyntaxGrammar, SyntaxVariable, VariableType,
    };
    #[pymodule_export]
    use crate::nfa::{CharacterSet, Nfa, NfaState};
    #[pymodule_export]
    use crate::node_types::{ChildQuantity, ChildType, FieldInfo, VariableInfo};
    #[pymodule_export]
    use crate::rules::{Alias, Associativity, Precedence, Symbol, SymbolType, TokenSet};
    #[pymodule_export]
    use crate::tables::{
        AdvanceAction, FieldLocation, GotoAction, LexState, LexTable, ParseAction, ParseState,
        ParseTable, ParseTableEntry, ProductionInfo,
    };

    #[pymodule_export]
    const START_PARSE_STATE_ID: usize = 1;
    #[pymodule_export]
    const EMPTY_EXTERNAL_LEX_STATE_ID: usize = 0;

    #[pyclass(frozen)]
    struct Grammar {
        #[pyo3(get)]
        syntax_grammar: SyntaxGrammar,
        #[pyo3(get)]
        lexical_grammar: LexicalGrammar,
        #[pyo3(get)]
        default_aliases: HashMap<Symbol, Alias>,
        #[pyo3(get)]
        tables: Tables,
        #[pyo3(get)]
        inlines: InlinedProductionMap,
        #[pyo3(get)]
        variable_info: Vec<VariableInfo>,
    }

    #[pyclass(frozen, from_py_object)]
    #[derive(Clone)]
    pub struct InlinedProductionMap {
        #[pyo3(get)]
        pub productions: Vec<Production>,
        #[pyo3(get)]
        pub production_map: HashMap<(Production, u32), Vec<usize>>,
    }

    #[pymethods]
    impl Alias {
        fn __repr__(&self) -> String {
            format!("Alias(value={}, is_named={})", self.value, self.is_named)
        }
    }

    #[pymethods]
    impl CharacterSet {
        #[getter]
        fn get_ranges(&self) -> Vec<(char, char)> {
            self.ranges().map(|r| (*r.start(), *r.end())).collect()
        }
    }
    #[pymethods]
    impl ExternalToken {
        fn __repr__(&self) -> String {
            format!("ExternalToken({:?})", self.name)
        }
    }

    #[pymethods]
    impl GotoAction {
        fn __repr__(&self) -> String {
            match self {
                GotoAction::Goto(state) => format!("Goto(state={})", state),
                GotoAction::ShiftExtra() => "ShiftExtra()".to_string(),
            }
        }
    }

    #[pymethods]
    impl Grammar {
        #[new]
        #[pyo3(signature = (path, *, extend_grammar, minimize_parser))]
        fn new(
            path: &str,
            extend_grammar: bool,
            minimize_parser: bool,
        ) -> Self {
            let grammar_json = fs::read_to_string(path).unwrap();
            let input_grammar = parse_grammar(&grammar_json).unwrap();

            let (mut syntax_grammar, mut lexical_grammar, inlines, default_aliases) =
                prepare_grammar(&input_grammar, extend_grammar).unwrap();

            rename_variables(&mut syntax_grammar, &mut lexical_grammar);

            let variable_info =
                get_variable_info(&syntax_grammar, &lexical_grammar, &default_aliases).unwrap();

            let tables = build_tables(
                &syntax_grammar,
                &lexical_grammar,
                &default_aliases,
                &variable_info,
                &inlines,
                None,
                minimize_parser,
            )
            .unwrap();

            Grammar {
                syntax_grammar: syntax_grammar,
                lexical_grammar: lexical_grammar,
                default_aliases: default_aliases,
                tables: tables,
                inlines: InlinedProductionMap {
                    productions: inlines.productions,
                    production_map: inlines
                        .production_map
                        .iter()
                        .map(|(k, v)| ((unsafe { (*k.0).clone() }, k.1), v.clone()))
                        .collect(),
                },
                variable_info: variable_info,
            }
        }
    }

    #[pymethods]
    impl LexicalVariable {
        fn __repr__(&self) -> String {
            format!("LexicalVariable({:?})", self.name)
        }
    }

    #[pymethods]
    impl ParseAction {
        fn __repr__(&self) -> String {
            match self {
                ParseAction::Accept() => "Accept()".to_string(),
                ParseAction::Shift {
                    state,
                    is_repetition,
                } => {
                    format!("Shift(state={}, is_repetition={})", state, is_repetition)
                }
                ParseAction::ShiftExtra() => "ShiftExtra()".to_string(),
                ParseAction::Recover() => "Recover()".to_string(),
                ParseAction::Reduce {
                    symbol,
                    child_count,
                    dynamic_precedence,
                    production_id,
                } => {
                    format!(
                        "Reduce(symbol={}, child_count={}, dynamic_precedence={}, production_id={})",
                        symbol.__repr__(), child_count, dynamic_precedence, production_id
                    )
                }
            }
        }
    }

    #[pymethods]
    impl ParseState {
        fn __repr__(&self) -> String {
            format!(
                "ParseState(id={}, lex_state_id={}, anti_lex_state_id={:?}, external_lex_state_id={}, core_id={})",
                self.id, self.lex_state_id, self.anti_lex_state_id, self.external_lex_state_id, self.core_id
            )
        }

        #[getter]
        fn get_terminal_entries(&self) -> HashMap<Symbol, ParseTableEntry> {
            self.terminal_entries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }

        #[getter]
        fn get_nonterminal_entries(&self) -> HashMap<Symbol, GotoAction> {
            self.nonterminal_entries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }
    }

    #[pymethods]
    impl Production {
        fn __repr__(&self) -> String {
            format!(
                "<Production: #steps={}, dynamic_precedence={}>",
                self.steps.len(),
                self.dynamic_precedence
            )
        }
    }

    #[pymethods]
    impl ProductionStep {
        fn __repr__(&self) -> String {
            format!("ProductionStep(symbol_kind={:?}, symbol_index={}, precedence={:?}, associativity={:?}, alias={:?}, field_name={:?}, reserved_word_set_id={})",
                self.symbol.kind, self.symbol.index, self.precedence, self.associativity, self.alias, self.field_name, self.reserved_word_set_id)
        }
    }

    #[pymethods]
    impl Symbol {
        fn __repr__(&self) -> String {
            format!("Symbol(kind={:?}, index={})", self.kind, self.index)
        }
    }

    #[pymethods]
    impl SyntaxVariable {
        fn __repr__(&self) -> String {
            format!("SyntaxVariable({:?})", self.name)
        }
    }

    #[pymethods]
    impl TokenSet {
        #[getter]
        fn get_tokens(&self) -> HashSet<Symbol> {
            self.iter().collect()
        }

        #[getter]
        fn get_terminals(&self) -> HashSet<Symbol> {
            self.terminals().collect()
        }
    }
}
