from typing import Final, NewType, Self

CoreId = NewType("CoreId", int)
ExternalLexStateId = NewType("ExternalLexStateId", int)
FieldLocationId = NewType("FieldLocationId", int)
LexicalVariableId = NewType("LexicalVariableId", int)
LexStateId = NewType("LexStateId", int)
NfaStateId = NewType("NfaStateId", int)
ParseStateId = NewType("ParseStateId", int)
ProductionId = NewType("ProductionId", int)
ReservedWordSetId = NewType("ReservedWordSetId", int)
StepId = NewType("StepId", int)

START_PARSE_STATE_ID: ParseStateId
EMPTY_EXTERNAL_LEX_STATE_ID: ExternalLexStateId

class AdvanceAction:
    state: Final[LexStateId]
    in_main_token: Final[bool]

class Alias:
    value: Final[str]
    is_named: Final[bool]
    def __hash__(self) -> int: ...

class Associativity:
    Left: Self
    Right: Self
    def __hash__(self) -> int: ...

class CharacterSet:
    ranges: Final[list[tuple[str, str]]]
    def __hash__(self) -> int: ...

class ChildType:
    class Normal(ChildType):
        __match_args__ = ("_0",)
        _0: Final[Symbol]

    class Aliased(ChildType):
        __match_args__ = ("_0",)
        _0: Final[Symbol]

    def __hash__(self) -> int: ...

class ChildQuantity:
    exists: Final[bool]
    required: Final[bool]
    multiple: Final[bool]
    def __hash__(self) -> int: ...

class ExternalToken:
    name: Final[str]
    kind: Final[VariableType]
    corresponding_internal_token: Final[Symbol | None]
    def __hash__(self) -> int: ...

class FieldInfo:
    quantity: Final[ChildQuantity]
    types: Final[list[ChildType]]

class FieldLocation:
    index: Final[FieldLocationId]
    inherited: Final[bool]
    def __hash__(self) -> int: ...

class GotoAction:
    class Goto(GotoAction):
        __match_args__ = ("_0",)
        _0: Final[ParseStateId]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class ShiftExtra(GotoAction):
        __match_args__ = ()
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    def __hash__(self) -> int: ...

class Grammar:
    syntax_grammar: Final[SyntaxGrammar]
    lexical_grammar: Final[LexicalGrammar]
    default_aliases: Final[dict[Symbol, Alias]]
    tables: Final[Tables]
    inlines: Final[InlinedProductionMap]
    variable_info: Final[list[VariableInfo]]

    def __init__(self, path: str, *, extend_grammar: bool, minimize_parser: bool) -> None: ...

class InlinedProductionMap:
    productions: Final[list[Production]]
    production_map: Final[dict[tuple[Production, StepId], list[ProductionId]]]

class LexicalGrammar:
    nfa: Final[Nfa]
    variables: Final[list[LexicalVariable]]

class LexicalVariable:
    name: Final[str]
    kind: Final[VariableType]
    implicit_precedence: Final[int]
    start_state: Final[LexStateId]
    def __hash__(self) -> int: ...

class LexState:
    accept_action: Final[Symbol | None]
    eof_action: Final[AdvanceAction | None]
    advance_actions: Final[list[tuple[CharacterSet, AdvanceAction]]]

class LexTable:
    states: Final[list[LexState]]

class Nfa:
    states: Final[list[NfaState]]

class NfaState:
    class Advance(NfaState):
        __match_args__ = ("chars", "state_id", "is_sep", "precedence")
        chars: Final[list[str]]
        state_id: Final[NfaStateId]
        is_sep: Final[bool]
        precedence: Final[int]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Split(NfaState):
        __match_args__ = ("_0", "_1")
        _0: Final[NfaStateId]
        _1: Final[NfaStateId]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Accept(NfaState):
        __match_args__ = ("variable_index", "precedence")
        variable_index: Final[LexicalVariableId]
        precedence: Final[Precedence]

    def __hash__(self) -> int: ...

class ParseAction:
    class Accept(ParseAction):
        __match_args__ = ()
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Recover(ParseAction):
        __match_args__ = ()
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Reduce(ParseAction):
        __match_args__ = (
            "symbol",
            "child_count",
            "dynamic_precedence",
            "production_id",
        )
        symbol: Final[Symbol]
        child_count: Final[int]
        dynamic_precedence: Final[Precedence]
        production_id: Final[ProductionId]

    class Shift(ParseAction):
        __match_args__ = ("state", "is_repetition")
        state: Final[ParseStateId]
        is_repetition: Final[bool]

    class ShiftExtra(ParseAction):
        __match_args__ = ()
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    def __hash__(self) -> int: ...

class ParseState:
    id: Final[ParseStateId]
    terminal_entries: Final[dict[Symbol, ParseTableEntry]]
    nonterminal_entries: Final[dict[Symbol, GotoAction]]
    reserved_words: Final[TokenSet]
    lex_state_id: Final[LexStateId]
    anti_lex_state_id: Final[LexStateId]
    external_lex_state_id: Final[ExternalLexStateId]
    core_id: Final[CoreId]

class ParseTable:
    states: Final[list[ParseState]]
    symbols: Final[list[Symbol]]
    production_infos: Final[list[ProductionInfo]]
    max_aliased_production_length: Final[int]
    external_lex_states: Final[list[TokenSet]]

class ParseTableEntry:
    actions: Final[list[ParseAction]]
    reusable: Final[bool]
    def __hash__(self) -> int: ...

class Precedence:
    class NoPrecedence(Precedence):
        __match_args__ = ()
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Integer(Precedence):
        __match_args__ = ("_0",)
        _0: Final[int]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> object: ...

    class Name(Precedence):
        __match_args__ = ("_0",)
        _0: Final[str]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> str: ...

    def __hash__(self) -> int: ...

class PrecedenceEntry:
    class Name(PrecedenceEntry):
        __match_args__ = ("_0",)
        _0: Final[str]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> str: ...

    class Symbol(PrecedenceEntry):
        __match_args__ = ("_0",)
        _0: Final[str]
        def __len__(self) -> int: ...
        def __getitem__(self, index: int, /) -> str: ...

    def __hash__(self) -> int: ...

class Production:
    steps: Final[list[ProductionStep]]
    dynamic_precedence: Final[Precedence]
    def __hash__(self) -> int: ...

class ProductionInfo:
    alias_sequence: Final[list[Alias | None]]
    field_map: Final[dict[str, list[FieldLocation]]]

class ProductionStep:
    symbol: Final[Symbol]
    precedence: Final[Precedence]
    associativity: Final[Associativity | None]
    alias: Final[Alias | None]
    field_name: Final[str | None]
    reserved_word_set_id: Final[ReservedWordSetId]
    def __hash__(self) -> int: ...

class SyntaxGrammar:
    variables: Final[list[SyntaxVariable]]
    extra_symbols: Final[list[Symbol]]
    expected_conflicts: Final[list[list[Symbol]]]
    external_tokens: Final[list[ExternalToken]]
    supertype_symbols: Final[list[Symbol]]
    variables_to_inline: Final[list[Symbol]]
    word_token: Final[Symbol | None]
    precedence_orderings: Final[list[list[PrecedenceEntry]]]
    reserved_word_sets: Final[list[TokenSet]]

class SyntaxVariable:
    name: Final[str]
    kind: Final[VariableType]
    productions: Final[list[Production]]
    def __hash__(self) -> int: ...

class Symbol:
    kind: Final[SymbolType]
    index: Final[int]
    def __hash__(self) -> int: ...

class SymbolType:
    External: Self
    End: Self
    EndOfNonTerminalExtra: Self
    Terminal: Self
    NonTerminal: Self
    def __hash__(self) -> int: ...

class Tables:
    parse_table: Final[ParseTable]
    main_lex_table: Final[LexTable]
    keyword_lex_table: Final[LexTable]
    large_character_sets: Final[list[tuple[Symbol | None, CharacterSet]]]

class TokenSet:
    tokens: Final[set[Symbol]]
    terminals: Final[set[Symbol]]
    eof: Final[bool]
    end_of_nonterminal_extra: Final[bool]
    def __hash__(self) -> int: ...

class VariableInfo:
    fields: Final[dict[str, FieldInfo]]
    children: Final[FieldInfo]
    children_without_fields: Final[FieldInfo]
    has_multi_step_production: Final[bool]

class VariableType:
    Hidden: Self
    Auxiliary: Self
    Anonymous: Self
    Named: Self
    def __hash__(self) -> int: ...
