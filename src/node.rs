use std::fmt::Debug;
use std::rc::Rc;

use typst_syntax::{parse, LinkedNode};

/// This function will give us the root node of our format tree.
///
/// We get rid of unneeded information in LinkedNode and store whatever fundamental information
/// we might need later here.
///
/// The pro of working on a tree is that we can apply transformations to it
///  (see (preserve_pass)[super::preserve_pass]) that will add formatting context.
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

#[derive(Clone)]
pub(crate) struct FmtNode<'a> {
    pub(crate) parent: Option<Rc<FmtNode<'a>>>,
    pub(crate) node: typst_syntax::LinkedNode<'a>,
    pub(crate) content: Content<'a>,
    pub(crate) kind: FmtKind,
}

impl<'a> FmtNode<'a> {
    // Todo, we could trust that it's a text and return "" otherwise
    // it should be considered a bug it it doesn't matches Text anyway.
    pub fn text(&self) -> String {
        match &self.content {
            Content::Text(s) => s.to_string(),
            Content::Children(c) => c
                .iter()
                .map(|c| c.text())
                .fold(String::new(), |a, b| format!("{a}{b}")),
        }
    }
}

impl Debug for FmtNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}\n", self.kind, self.content)
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

/// All the kinds that need to be handled differently while formatting

// todo a LetBinding, ForLoop, WhileLoop node forcing a linebreak in code mode or a ; in markup?
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FmtKind {
    // used only for when in preserve Node, ignored otherwise.
    FuncCall,
    ParamsLike,
    ParamsLikeParenthesized,
    Binary,
    Unary,
    /// The content must be preserved as is.
    ///
    /// This can contain a Raw block or anything targeted in the
    /// (preserve_pass)[super::preserve_pass]
    Preserve(i32),
    Space,
    Parbreak,
    /// Common case that doesn't need precise logic, might be ignored if it is the
    /// child of a custom logic Node such as one with kind : Node.
    WithSpacing(Spacing),

    Markup,
    ContentBlock,
    OneLineMarkup,
    /// if else chain that we must format all at the same time for maximum style.
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
            content: Content::Children(vec![]),
            kind: self,
            parent,
            node: node.clone(),
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
            content: Content::Text(node.get().text()),
            kind: self,
            parent,
            node,
        }
    }
}

#[test]
fn feature() {
    let s = "#[ text ]";
    let node = parse(s);

    let root = LinkedNode::new(&node);
    let aft = map_tree(root.clone(), None);


    dbg!(&node);
    dbg!(&root);
    dbg!(aft);
}
