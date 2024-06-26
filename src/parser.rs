use chumsky::{input::ValueInput, prelude::*};

pub type Span = SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Ctrl(char),
    Op(&'src str),
    Ident(&'src str),
    Comment(&'src str),
    Other(char),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Groups parsed information about a field of a struct.
#[derive(Clone, Debug, PartialEq)]
pub struct StructField<'src> {
    pub name: &'src str,
    pub docs: String,
}

/// Groups parsed information about a struct.
#[derive(Clone, Debug, PartialEq)]
pub struct StructInfo<'src> {
    pub name: &'src str,
    pub fields: Vec<StructField<'src>>,
    pub docs: String,
}

/// Groups parsed information about an enum.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumInfo<'src> {
    pub name: &'src str,
    pub docs: String,
}

/// Groups parsed information about a trait.
#[derive(Clone, Debug, PartialEq)]
pub struct TraitInfo<'src> {
    pub name: &'src str,
    pub docs: String,
}

/// Groups parsed information about a const value.
#[derive(Clone, Debug, PartialEq)]
pub struct ConstInfo<'src> {
    pub name: &'src str,
    pub docs: String,
}

/// Groups parsed information about a function.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInfo<'src> {
    pub name: &'src str,
    pub args: Vec<&'src str>,
    pub void_return_type: bool,
    pub docs: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComplexToken<'src> {
    Struct(StructInfo<'src>),
    Function(FunctionInfo<'src>),
    Enum(EnumInfo<'src>),
    Trait(TraitInfo<'src>),
    Const(ConstInfo<'src>),
    Other(Token<'src>),
}

impl std::fmt::Display for ComplexToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn token_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    // Parser for operators.
    let operator = just("->").map(|s: &str| Token::Op(s));

    // A parser for control characters (delimiters, semicolons, etc.)
    let ctrl = one_of("(),:{}<>").map(Token::Ctrl);

    // A parser for identifiers.
    let ident = text::ascii::ident().map(|ident: &str| Token::Ident(ident));

    // Parsers for comments.
    let simple_comment = just("//")
        .or(just("///"))
        .ignore_then(
            any()
                .and_is(just("\n").not())
                .repeated()
                .to_slice()
                .padded(),
        )
        .map(Token::Comment);
    let c_comment = just("/**")
        .or(just("/*!"))
        .ignore_then(any().and_is(just("*/").not()).repeated().to_slice())
        .then_ignore(just("*/"))
        .map(Token::Comment);
    let comment = c_comment.or(simple_comment);

    // A single token can be one of the above.
    let token = comment
        .or(operator)
        .or(ctrl)
        .or(ident)
        .or(any().map(Token::Other));

    token
        .map_with(|t, extra| (t, extra.span()))
        .padded()
        .repeated()
        .collect()
}

pub fn complex_token_parser<'src, I>(
) -> impl Parser<'src, I, Vec<(ComplexToken<'src>, Span)>, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    let ident = select! { Token::Ident(ident) => ident };
    let comment = select! { Token::Comment(c) => c};
    let token = select! { token => token };

    // Parsers for simple types.
    let simple_type_no_ref_parser = any().and_is(ident).or(just(Token::Other('['))
        .then_ignore(ident)
        .then_ignore(just(Token::Other(']'))));
    let simple_type_ref_parser = just(Token::Other('&'))
        .then_ignore(just(Token::Other('\'')).then_ignore(ident).or_not())
        .then_ignore(just(Token::Ident("mut")).or_not())
        .then_ignore(simple_type_no_ref_parser.clone());
    let simple_type_mut_parser =
        just(Token::Ident("mut")).then_ignore(simple_type_no_ref_parser.clone());
    let simple_type_trait_parser = just(Token::Other('&'))
        .then_ignore(just(Token::Ident("impl")))
        .then_ignore(simple_type_no_ref_parser.clone());
    let simple_type_dyn_trait_parser =
        just(Token::Ident("dyn")).then_ignore(simple_type_no_ref_parser.clone());

    let simple_type_parser = simple_type_dyn_trait_parser
        .or(simple_type_trait_parser)
        .or(simple_type_mut_parser)
        .or(simple_type_ref_parser)
        .or(simple_type_no_ref_parser);

    // A parser for tuple types.
    let tuple_type_parser = recursive(|parser_copy| {
        simple_type_parser
            .clone()
            .or(just(Token::Ctrl('(')).then_ignore(simple_type_parser.clone()))
            .then_ignore(just(Token::Ctrl(')')).or(just(Token::Ctrl(',')).then_ignore(parser_copy)))
    });

    // A parser for generic types.
    let generic_inner_parser = recursive(|parser_copy| {
        just(Token::Ctrl('<'))
            .or_not()
            .then_ignore(tuple_type_parser.clone().or(simple_type_parser.clone()))
            .then_ignore(
                just(Token::Ctrl('<'))
                    .then_ignore(parser_copy.clone())
                    .or_not(),
            )
            .then_ignore(just(Token::Ctrl('>')).or(just(Token::Ctrl(',')).then_ignore(parser_copy)))
    });
    let generic_type_parser = simple_type_parser.clone().then_ignore(generic_inner_parser);

    // A parser for types.
    let type_parser = generic_type_parser
        .or(tuple_type_parser)
        .or(simple_type_parser.clone());

    // A parser for attributes (like #[derive(...)]).
    let attribute_parser = just(Token::Other('#'))
        .then_ignore(just(Token::Other('[')))
        .then_ignore(any().and_is(just(Token::Other(']')).not()).repeated())
        .then_ignore(just(Token::Other(']')));

    // A parser for struct fields.
    let field = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(attribute_parser.clone().repeated())
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then(ident) // name
        .then_ignore(just(Token::Ctrl(':')))
        .then_ignore(type_parser.clone())
        .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl('}'))).or_not())
        .map(|(opt_comments, name)| StructField {
            name,
            docs: opt_comments.concat(),
        });

    // A parser for structs.
    let struct_parser = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(attribute_parser.clone().repeated())
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then_ignore(just(Token::Ident("struct")))
        .then(ident) // name
        .then_ignore(any().and_is(just(Token::Ctrl('{')).not()).repeated()) // skip any generics/lifetimes
        .then_ignore(just(Token::Ctrl('{')))
        .then(field.repeated().collect())
        .then_ignore(just(Token::Ctrl('}')).or_not())
        .map(|((opt_comments, name), fields)| {
            ComplexToken::Struct(StructInfo {
                name,
                fields,
                docs: opt_comments.concat(),
            })
        });

    // A parser for enums.
    let enum_parser = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(attribute_parser.clone().repeated())
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then_ignore(just(Token::Ident("enum")))
        .then(ident) // name
        .map(|(opt_comments, name)| {
            ComplexToken::Enum(EnumInfo {
                name,
                docs: opt_comments.concat(),
            })
        });

    // A parser for traits.
    let trait_parser = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then_ignore(just(Token::Ident("trait")))
        .then(ident) // name
        .map(|(opt_comments, name)| {
            ComplexToken::Trait(TraitInfo {
                name,
                docs: opt_comments.concat(),
            })
        });

    // A parser for const values.
    let const_parser = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(attribute_parser.clone().repeated())
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then_ignore(just(Token::Ident("const")))
        .then(ident) // name
        .map(|(opt_comments, name)| {
            ComplexToken::Const(ConstInfo {
                name,
                docs: opt_comments.concat(),
            })
        });

    // A parser for function arguments.
    let non_self_func_argument = just(Token::Ident("mut"))
        .ignore_then(ident)
        .or(ident)
        .then_ignore(just(Token::Ctrl(':')))
        .then_ignore(type_parser)
        .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl(')'))).or_not())
        .map(|name| name);

    let self_func_argument = just(Token::Ident("self"))
        .or(just(Token::Other('&')).then_ignore(just(Token::Ident("self"))))
        .or(just(Token::Other('&'))
            .then_ignore(just(Token::Ident("mut")))
            .then_ignore(just(Token::Ident("self"))))
        .then_ignore(just(Token::Ctrl(',')).or(just(Token::Ctrl(')'))))
        .ignored()
        .map(|()| "self");

    // A parser for function arguments.
    let func_argument = self_func_argument.or(non_self_func_argument);

    // A parser for functions.
    let function = comment
        .repeated()
        .collect::<Vec<&str>>()
        .then_ignore(
            just(Token::Ident("extern"))
                .then_ignore(just(Token::Other('"')))
                .then_ignore(ident)
                .then_ignore(just(Token::Other('"')))
                .or_not(),
        )
        .then_ignore(attribute_parser.repeated())
        .then_ignore(just(Token::Ident("pub")).or_not())
        .then_ignore(just(Token::Ident("const")).or_not())
        .then_ignore(just(Token::Ident("unsafe")).or_not())
        .then_ignore(just(Token::Ident("fn")))
        .then(ident)
        .then_ignore(any().and_is(just(Token::Ctrl('(')).not()).repeated()) // skip any generics/lifetimes
        .then_ignore(just(Token::Ctrl('(')))
        .then(func_argument.clone().repeated().collect())
        .then_ignore(just(Token::Ctrl(')')).or_not())
        .then(just(Token::Op("->")).or_not())
        .map(|(((opt_comments, name), args), opt_return)| {
            ComplexToken::Function(FunctionInfo {
                name,
                args,
                void_return_type: opt_return.is_none(),
                docs: opt_comments.concat(),
            })
        });

    // If non of our parsers from above worked then just pass the token.
    let output = function
        .or(struct_parser)
        .or(enum_parser)
        .or(const_parser)
        .or(trait_parser)
        .or(token.map(ComplexToken::Other));

    output
        .map_with(|t, extra| (t, extra.span()))
        .repeated()
        .collect()
}
