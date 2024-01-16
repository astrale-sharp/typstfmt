use std::fmt::Debug;
use std::rc::Rc;

pub(crate) fn map_tree<'a>(
    node: typst_syntax::LinkedNode<'a>,
    parent: Option<Rc<FmtNode<'a>>>,
) -> FmtNode<'a> {
    use typst_syntax::SyntaxKind::{self, *};

    match node.kind() {
        Raw => FmtKind::Preserve(1).with_text(node, parent),
        Space => FmtKind::Space.with_text(node, parent),
        Linebreak => FmtKind::WithSpacing(Spacing::StrongBrkLine).with_text(node, parent),
        Parbreak => FmtKind::Parbreak.with_text(node, parent),
        Markup => FmtKind::Markup.with_children(node, parent),
        Code => FmtKind::Code.with_children(node, parent),
        CodeBlock => FmtKind::CodeBlock.with_children(node, parent),
        ContentBlock => FmtKind::ContentBlock.with_children(node, parent),
        Math => FmtKind::Math.with_children(node, parent),
        Equation => FmtKind::Equation.with_children(node, parent),
        FuncCall => FmtKind::FuncCall.with_children(node, parent),
        Array | Dict | Args | ListItem | EnumItem | TermItem | Params | Destructuring => {
            FmtKind::ParamsLike.with_children(node, parent)
        }
        Parenthesized => FmtKind::ParamsLikeParenthesized.with_children(node, parent),
        Heading => FmtKind::OneLineMarkup.with_children(node, parent),
        Unary => FmtKind::Unary.with_children(node, parent),
        Binary => FmtKind::Binary.with_children(node, parent),

        LetBinding | ModuleImport | ImportItems | ModuleInclude | SetRule | ShowRule => {
            FmtKind::WithSpacing(Spacing::StrongBrkLine).with_children(node, parent)
        }

        Eof | Spread | Root | Dots | Dot | Star | Underscore | Hashtag | RefMarker | SmartQuote => {
            FmtKind::WithSpacing(Spacing::Destruct).with_text(node, parent)
        }

        LoopBreak
        | LoopContinue
        | FuncReturn
        | Bool
        | Int
        | Float
        | Numeric
        | Str
        | Arrow
        | Not
        | And
        | Or
        | SyntaxKind::None
        | Auto
        | Let
        | Set
        | Show
        | If
        | Else
        | For
        | In
        | While
        | Break
        | Continue
        | Return
        | Import
        | Include
        | As
        | Eq
        | EqEq
        | ExclEq
        | Lt
        | LtEq
        | Gt
        | GtEq
        | PlusEq
        | HyphEq
        | StarEq
        | SlashEq
        | Plus
        | Minus
        | Hat
        | Prime
        | Slash
        | Semicolon
        | Dollar
        | LeftParen
        | RightParen
        | LeftBrace
        | RightBrace
        | LeftBracket
        | RightBracket
        | MathAlignPoint
        | Error
        | Text
        | Shorthand
        | Escape
        | MathIdent => FmtKind::WithSpacing(Spacing::WeakSpace).with_text(node, parent),

        DestructAssignment | WhileLoop | ForLoop | Closure | FieldAccess | Named | Keyed
        | MathFrac | MathRoot | MathAttach | Link | Label | Strong | Emph => {
            FmtKind::WithSpacing(Spacing::WeakSpace).with_children(node, parent)
        }

        MathDelimited | ListMarker | EnumMarker | TermMarker | HeadingMarker | Ref => {
            FmtKind::WithSpacing(Spacing::WeakSpace).with_children(node, parent)
        }

        Ident | Colon | Comma | MathPrimes => {
            FmtKind::WithSpacing(Spacing::StrongSpace).with_text(node, parent)
        }

        Conditional => FmtKind::Conditional.with_children(node, parent),

        BlockComment | LineComment => FmtKind::Comment.with_text(node, parent),
    }
}

/// We translate the syntax tree, simplifying it for the
/// purpose of formatting only, then we apply rules to it to simplify
/// the formatting part.
///
/// Rules :
/// - Preserve: Raw blocks and area delimited by `// fmt::off` and `// fmt::on` shouldn't be formatted.
/// - Condition: We collapse all nested conditional nodes into one in
/// order to be able to format consistently across long if else chains.
/// - Binary: like conditional for for binary operation.
#[derive(Clone)]
pub(crate) struct FmtNode<'a> {
    parent: Option<Rc<FmtNode<'a>>>,
    pub(crate) content: Content<'a>,
    pub(crate) kind: FmtKind,
}

impl<'a> FmtNode<'a> {
    pub fn text(&self) -> Option<&'a str> {
        match self.content {
            Content::Text(s) => Some(s),
            Content::Children(_) => None,
        }
    }
}

impl Debug for FmtNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.kind, self.content)
    }
}

#[derive(Clone)]
pub(crate) enum Content<'a> {
    Text(&'a str),
    Children(Vec<FmtNode<'a>>),
}

impl Debug for Content<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Children(arg0) => arg0.fmt(f),
        }
    }
}

// todo a LetBinding, ForLoop, WhileLoop node forcing a linebreak in code mode or a ; in markup?
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FmtKind {
    // used only for when in preserve Node, ignored otherwise.
    FuncCall,
    ParamsLike,
    ParamsLikeParenthesized,
    Binary,
    Unary,
    /// We must preserve the content as is
    ///
    /// We smartly only tag the text nodes in the preserve_pass, so we can rely on it when
    /// `evaluating Preserve`
    /// TODO: panic if the node has children
    Preserve(i32),
    /// if else chain that we must format all at the same time for maximum style
    Space,
    Parbreak,

    WithSpacing(Spacing),

    Markup,
    ContentBlock,
    OneLineMarkup,
    Conditional,

    // todo: force line break after loops
    Code,
    CodeBlock,

    Math,
    Equation,

    Comment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Spacing {
    WeakSpace,
    Destruct,
    StrongSpace,
    StrongBrkLine,
}

impl FmtKind {
    pub(crate) fn with_children<'a>(
        self,
        node: typst_syntax::LinkedNode<'a>,
        parent: Option<Rc<FmtNode<'a>>>,
    ) -> FmtNode<'a> {
        let mut fnode = FmtNode {
            parent,
            content: Content::Children(vec![]),
            kind: self,
        };
        fnode.content = Content::Children(
            node.children()
                .map(|c| map_tree(c, Some(Rc::new(fnode.clone()))))
                .collect(),
        );

        fnode
    }

    pub(crate) fn with_text<'a>(
        self,
        node: typst_syntax::LinkedNode<'a>,
        parent: Option<Rc<FmtNode<'a>>>,
    ) -> FmtNode<'a> {
        FmtNode {
            parent,
            content: Content::Text(node.get().text()),

            kind: self,
        }
    }
}
