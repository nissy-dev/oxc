use std::rc::Rc;

use enum_as_inner::EnumAsInner;
use oxc_ast::{
    ast::{
        BindingPatternKind, Class, Expression, IdentifierName, IdentifierReference, IfStatement,
        ImportDeclaration, ImportDefaultSpecifier, ImportSpecifier, JSXAttribute, JSXElement,
        JSXExpressionContainer, JSXFragment, JSXOpeningElement, JSXSpreadAttribute, JSXSpreadChild,
        JSXText, MemberExpression, MethodDefinition, ModuleDeclaration, NumberLiteral,
        ObjectExpression, ObjectProperty, PropertyDefinition, ReturnStatement, SpreadElement,
        StaticMemberExpression, TSInterfaceDeclaration, TSType, TSTypeAnnotation,
        VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{GetSpan, Span};
use trustfall::provider::Typename;
use url::Url;

use crate::util::{expr_to_maybe_const_string, jsx_attribute_to_constant_string};

#[non_exhaustive]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Vertex<'a> {
    ASTNode(AstNode<'a>),
    AssignmentType(&'a BindingPatternKind<'a>),
    Class(Rc<ClassVertex<'a>>),
    ClassMethod(Rc<ClassMethodVertex<'a>>),
    ClassProperty(Rc<ClassPropertyVertex<'a>>),
    DefaultImport(&'a ImportDefaultSpecifier),
    Expression(&'a Expression<'a>),
    File,
    Import(Rc<ImportVertex<'a>>),
    Interface(Rc<InterfaceVertex<'a>>),
    InterfaceExtend(Rc<InterfaceExtendVertex<'a>>),
    JSXAttribute(&'a JSXAttribute<'a>),
    JSXElement(Rc<JSXElementVertex<'a>>),
    JSXExpressionContainer(&'a JSXExpressionContainer<'a>),
    JSXFragment(&'a JSXFragment<'a>),
    JSXOpeningElement(Rc<JSXOpeningElementVertex<'a>>),
    JSXSpreadAttribute(&'a JSXSpreadAttribute<'a>),
    JSXSpreadChild(&'a JSXSpreadChild<'a>),
    JSXText(&'a JSXText),
    ObjectLiteral(Rc<ObjectLiteralVertex<'a>>),
    NumberLiteral(Rc<NumberLiteralVertex<'a>>),
    Name(Rc<NameVertex<'a>>),
    PathPart(usize),
    SearchParameter(Rc<SearchParameterVertex>),
    Span(Span),
    SpecificImport(&'a ImportSpecifier),
    TypeAnnotation(Rc<TypeAnnotationVertex<'a>>),
    Type(&'a TSType<'a>),
    Url(Rc<Url>),
    VariableDeclaration(Rc<VariableDeclarationVertex<'a>>),
    ReturnStatementAST(Rc<ReturnStatementVertex<'a>>),
    IfStatementAST(Rc<IfStatementVertex<'a>>),
    SpreadIntoObject(Rc<SpreadIntoObjectVertex<'a>>),
    ObjectEntry(Rc<ObjectEntryVertex<'a>>),
    DotProperty(Rc<DotPropertyVertex<'a>>),
}

impl<'a> Vertex<'a> {
    pub fn span(&self) -> Span {
        match &self {
            Self::AssignmentType(data) => data.span(),
            Self::ASTNode(data) => data.kind().span(),
            Self::Class(data) => data.class.span,
            Self::ClassMethod(data) => data.method.span,
            Self::ClassProperty(data) => data.property.span,
            Self::DefaultImport(data) => data.span,
            Self::Expression(data) => data.span(),
            Self::Import(data) => data.import.span,
            Self::Interface(data) => data.interface.span,
            Self::InterfaceExtend(data) => match **data {
                InterfaceExtendVertex::Identifier(ident) => ident.span,
                InterfaceExtendVertex::MemberExpression(membexpr) => (*membexpr).span(),
            },
            Self::JSXAttribute(data) => data.span,
            Self::JSXElement(data) => data.element.span,
            Self::JSXExpressionContainer(data) => data.span,
            Self::JSXFragment(data) => data.span,
            Self::JSXOpeningElement(data) => data.opening_element.span,
            Self::DotProperty(data) => data.static_member_expr.span,
            Self::JSXSpreadAttribute(data) => data.span,
            Self::JSXSpreadChild(data) => data.span,
            Self::JSXText(data) => data.span,
            Self::ObjectLiteral(data) => data.object_expression.span,
            Self::SpreadIntoObject(data) => data.property.span,
            Self::ObjectEntry(data) => data.property.span,
            Self::SpecificImport(data) => data.span,
            Self::TypeAnnotation(data) => data.type_annotation.span,
            Self::Type(data) => data.span(),
            Self::VariableDeclaration(data) => data.variable_declaration.span,
            Self::ReturnStatementAST(data) => data.return_statement.span,
            Self::IfStatementAST(data) => data.return_statement.span,
            Self::NumberLiteral(data) => data.number_literal.span,
            Self::Name(data) => data.name.span,
            Self::File
            | Self::Url(_)
            | Self::PathPart(_)
            | Self::SearchParameter(_)
            | Self::Span(_) => {
                unreachable!("Tried to get the span from a {self:#?}")
            }
        }
    }

    pub fn ast_node_id(&self) -> Option<AstNodeId> {
        match &self {
            Vertex::ASTNode(data) => Some(data.id()),
            Vertex::Class(data) => data.ast_node.map(|x| x.id()),
            Vertex::Import(data) => data.ast_node.map(|x| x.id()),
            Vertex::Interface(data) => data.ast_node.map(|x| x.id()),
            Vertex::JSXElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::TypeAnnotation(data) => data.ast_node.map(|x| x.id()),
            Vertex::VariableDeclaration(data) => data.ast_node.map(|x| x.id()),
            Vertex::ObjectLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::ReturnStatementAST(data) => data.ast_node.map(|x| x.id()),
            Vertex::IfStatementAST(data) => data.ast_node.map(|x| x.id()),
            Vertex::JSXOpeningElement(data) => data.ast_node.map(|x| x.id()),
            Vertex::NumberLiteral(data) => data.ast_node.map(|x| x.id()),
            Vertex::Name(data) => data.ast_node.map(|x| x.id()),
            Vertex::SpreadIntoObject(data) => data.ast_node.map(|x| x.id()),
            Vertex::ObjectEntry(data) => data.ast_node.map(|x| x.id()),
            Vertex::DotProperty(data) => data.ast_node.map(|x| x.id()),
            Vertex::DefaultImport(_)
            | Vertex::AssignmentType(_)
            | Vertex::ClassMethod(_)
            | Vertex::Expression(_)
            | Vertex::File
            | Vertex::InterfaceExtend(_)
            | Vertex::JSXAttribute(_)
            | Vertex::JSXExpressionContainer(_)
            | Vertex::JSXFragment(_)
            | Vertex::JSXText(_)
            | Vertex::JSXSpreadChild(_)
            | Vertex::JSXSpreadAttribute(_)
            | Vertex::PathPart(_)
            | Vertex::Url(_)
            | Vertex::Type(_)
            | Vertex::SpecificImport(_)
            | Vertex::Span(_)
            | Vertex::SearchParameter(_)
            | Vertex::ClassProperty(_) => None,
        }
    }

    pub fn make_url(attr: &'a JSXAttribute<'a>) -> Option<Self> {
        jsx_attribute_to_constant_string(attr)
            .as_deref()
            .and_then(|v| Url::parse(v).ok())
            .map(Rc::new)
            .map(Vertex::Url)
    }

    pub fn as_constant_string(&self) -> Option<String> {
        match &self {
            Vertex::Expression(expr) => expr_to_maybe_const_string(expr),
            _ => None,
        }
    }
}

impl Typename for Vertex<'_> {
    fn typename(&self) -> &'static str {
        match self {
            Vertex::ASTNode(_) => "ASTNode",
            Vertex::AssignmentType(_) => "AssignmentType",
            Vertex::Class(class) => class.typename(),
            Vertex::ClassMethod(_) => "ClassMethod",
            Vertex::ClassProperty(_) => "ClassProperty",
            Vertex::DefaultImport(_) => "DefaultImport",
            Vertex::Expression(_) => "Expression",
            Vertex::File => "File",
            Vertex::Import(import) => import.typename(),
            Vertex::Interface(iface) => iface.typename(),
            Vertex::NumberLiteral(nlit) => nlit.typename(),
            Vertex::DotProperty(dot_property) => dot_property.typename(),
            Vertex::InterfaceExtend(iex) => match **iex {
                InterfaceExtendVertex::Identifier(_) => "SimpleExtend",
                InterfaceExtendVertex::MemberExpression(_) => "MemberExtend",
            },
            Vertex::JSXAttribute(_) => "JSXAttribute",
            Vertex::JSXElement(jsx) => jsx.typename(),
            Vertex::JSXExpressionContainer(_) => "JSXExpressionContainer",
            Vertex::JSXFragment(_) => "JSXFragment",
            Vertex::JSXOpeningElement(jsx) => jsx.typename(),
            Vertex::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
            Vertex::JSXSpreadChild(_) => "JSXSpreadChild",
            Vertex::JSXText(_) => "JSXText",
            Vertex::ObjectLiteral(objlit) => objlit.typename(),
            Vertex::PathPart(_) => "PathPart",
            Vertex::SearchParameter(_) => "SearchParameter",
            Vertex::Span(_) => "Span",
            Vertex::SpecificImport(_) => "SpecificImport",
            Vertex::TypeAnnotation(tn) => tn.typename(),
            Vertex::Type(_) => "Type",
            Vertex::Url(_) => "URL",
            Vertex::VariableDeclaration(vd) => vd.typename(),
            Vertex::Name(name) => name.typename(),
            Vertex::ReturnStatementAST(_) => "ReturnStatementAST",
            Vertex::IfStatementAST(_) => "IfStatementAST",
            Vertex::SpreadIntoObject(obj) => obj.typename(),
            Vertex::ObjectEntry(entry) => entry.typename(),
        }
    }
}

impl<'a> From<AstNode<'a>> for Vertex<'a> {
    fn from(ast_node: AstNode<'a>) -> Self {
        match ast_node.kind() {
            AstKind::ReturnStatement(return_statement) => Self::ReturnStatementAST(
                ReturnStatementVertex { ast_node: Some(ast_node), return_statement }.into(),
            ),
            AstKind::IfStatement(if_statement) => Self::IfStatementAST(
                IfStatementVertex { ast_node: Some(ast_node), return_statement: if_statement }
                    .into(),
            ),
            AstKind::JSXElement(element) => {
                Self::JSXElement(JSXElementVertex { ast_node: Some(ast_node), element }.into())
            }
            AstKind::TSInterfaceDeclaration(interface) => {
                Self::Interface(InterfaceVertex { ast_node: Some(ast_node), interface }.into())
            }
            AstKind::TSTypeAnnotation(type_annotation) => Self::TypeAnnotation(
                TypeAnnotationVertex { ast_node: Some(ast_node), type_annotation }.into(),
            ),
            AstKind::VariableDeclarator(variable_declaration) => Self::VariableDeclaration(
                VariableDeclarationVertex { ast_node: Some(ast_node), variable_declaration }.into(),
            ),
            AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import)) => {
                Self::Import(ImportVertex { ast_node: Some(ast_node), import }.into())
            }
            AstKind::Class(class) => {
                Self::Class(ClassVertex { ast_node: Some(ast_node), class }.into())
            }
            AstKind::ObjectExpression(objexpr) => Self::ObjectLiteral(
                ObjectLiteralVertex { ast_node: Some(ast_node), object_expression: objexpr }.into(),
            ),
            AstKind::JSXOpeningElement(opening_element) => Self::JSXOpeningElement(
                JSXOpeningElementVertex { ast_node: Some(ast_node), opening_element }.into(),
            ),
            AstKind::NumberLiteral(number_literal) => Self::NumberLiteral(
                NumberLiteralVertex { ast_node: Some(ast_node), number_literal }.into(),
            ),
            AstKind::IdentifierName(identifier_name) => {
                Self::Name(NameVertex { ast_node: Some(ast_node), name: identifier_name }.into())
            }
            AstKind::ObjectProperty(property) => {
                Self::ObjectEntry(ObjectEntryVertex { ast_node: Some(ast_node), property }.into())
            }
            AstKind::SpreadElement(property) => Self::SpreadIntoObject(
                SpreadIntoObjectVertex { ast_node: Some(ast_node), property }.into(),
            ),
            AstKind::MemberExpression(member_expr)
                if matches!(member_expr, MemberExpression::StaticMemberExpression(_)) =>
            {
                match member_expr {
                    MemberExpression::StaticMemberExpression(member_expr) => Self::DotProperty(
                        DotPropertyVertex {
                            ast_node: Some(ast_node),
                            static_member_expr: member_expr,
                        }
                        .into(),
                    ),
                    _ => unreachable!("we should only ever have StaticMemberExpression"),
                }
            }
            _ => Vertex::ASTNode(ast_node),
        }
    }
}

impl<'a> From<&'a Expression<'a>> for Vertex<'a> {
    fn from(expr: &'a Expression<'a>) -> Self {
        // FIXME: We just get rid of all parentheses here, but we shouldn't do that...

        // NOTE: When string literal / template literal is added, add to as_constant_string
        match &expr.get_inner_expression() {
            Expression::ObjectExpression(object_expression) => Vertex::ObjectLiteral(
                ObjectLiteralVertex { ast_node: None, object_expression }.into(),
            ),
            Expression::JSXElement(element) => {
                Vertex::JSXElement(JSXElementVertex { ast_node: None, element }.into())
            }
            Expression::NumberLiteral(number_literal) => {
                Vertex::NumberLiteral(NumberLiteralVertex { ast_node: None, number_literal }.into())
            }
            Expression::MemberExpression(member_expr)
                if matches!(**member_expr, MemberExpression::StaticMemberExpression(_)) =>
            {
                match &**member_expr {
                    MemberExpression::StaticMemberExpression(static_member_expr) => {
                        Vertex::DotProperty(
                            DotPropertyVertex { ast_node: None, static_member_expr }.into(),
                        )
                    }
                    _ => unreachable!("we should only ever have StaticMemberExpression"),
                }
            }
            _ => Vertex::Expression(expr),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub class: &'a Class<'a>,
}

impl<'a> Typename for ClassVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ClassAST"
        } else {
            "Class"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassMethodVertex<'a> {
    pub method: &'a MethodDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ClassPropertyVertex<'a> {
    pub property: &'a PropertyDefinition<'a>,
    pub is_abstract: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ImportVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub import: &'a ImportDeclaration<'a>,
}

impl<'a> Typename for ImportVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ImportAST"
        } else {
            "Import"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct InterfaceVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub interface: &'a TSInterfaceDeclaration<'a>,
}

impl<'a> Typename for InterfaceVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "InterfaceAST"
        } else {
            "Interface"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InterfaceExtendVertex<'a> {
    Identifier(&'a IdentifierReference),
    MemberExpression(&'a MemberExpression<'a>),
}

impl<'a> From<&'a Expression<'a>> for InterfaceExtendVertex<'a> {
    fn from(expr: &'a Expression<'a>) -> Self {
        match &expr {
            Expression::Identifier(ident) => InterfaceExtendVertex::Identifier(ident),
            Expression::MemberExpression(membexpr) => {
                InterfaceExtendVertex::MemberExpression(membexpr)
            }
            _ => unreachable!(
                "Only ever possible to have an interface extend an identifier or memberexpr. see TS:2499"
            ),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct JSXElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub element: &'a JSXElement<'a>,
}

impl<'a> Typename for JSXElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "JSXElementAST"
        } else {
            "JSXElement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct IfStatementVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a IfStatement<'a>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ReturnStatementVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub return_statement: &'a ReturnStatement<'a>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TypeAnnotationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub type_annotation: &'a TSTypeAnnotation<'a>,
}

impl<'a> Typename for TypeAnnotationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "TypeAnnotationAST"
        } else {
            "TypeAnnotation"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SearchParameterVertex {
    pub key: String,
    pub value: String,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VariableDeclarationVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub variable_declaration: &'a VariableDeclarator<'a>,
}

impl<'a> Typename for VariableDeclarationVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "VariableDeclarationAST"
        } else {
            "VariableDeclaration"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ObjectLiteralVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub object_expression: &'a ObjectExpression<'a>,
}

impl<'a> Typename for ObjectLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ObjectLiteralAST"
        } else {
            "ObjectLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct JSXOpeningElementVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub opening_element: &'a JSXOpeningElement<'a>,
}

impl<'a> Typename for JSXOpeningElementVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "JSXOpeningElementAST"
        } else {
            "JSXOpeningElement"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NumberLiteralVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub number_literal: &'a NumberLiteral<'a>,
}

impl<'a> Typename for NumberLiteralVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "NumberLiteralAST"
        } else {
            "NumberLiteral"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NameVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub name: &'a IdentifierName,
}

impl<'a> Typename for NameVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "NameAST"
        } else {
            "Name"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ObjectEntryVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub property: &'a ObjectProperty<'a>,
}

impl<'a> Typename for ObjectEntryVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "ObjectEntryAST"
        } else {
            "ObjectEntry"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SpreadIntoObjectVertex<'a> {
    pub ast_node: Option<AstNode<'a>>,
    pub property: &'a SpreadElement<'a>,
}

impl<'a> Typename for SpreadIntoObjectVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "SpreadIntoObjectAST"
        } else {
            "SpreadIntoObject"
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct DotPropertyVertex<'a> {
    ast_node: Option<AstNode<'a>>,
    pub static_member_expr: &'a StaticMemberExpression<'a>,
}

impl<'a> Typename for DotPropertyVertex<'a> {
    fn typename(&self) -> &'static str {
        if self.ast_node.is_some() {
            "DotPropertyAST"
        } else {
            "DotProperty"
        }
    }
}
