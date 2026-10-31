
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

sema_old = """pub struct SemanticAnalyzer {
    pub scopes: Vec<Scope>,
    pub functions: HashMap<String, Function>,
    pub structs: HashMap<String, StructDecl>,
    pub enums: HashMap<String, EnumDecl>,
    pub extern_fns: HashMap<String, ExternDecl>,
    pub errors: Vec<SemaError>,
    pub warnings: Vec<SemaWarning>,
    pub current_fn_return_type: Option<Type>,
    pub last_return_type: Option<Type>,
    pub type_map: HashMap<Span, Type>,
    is_borrow_context: bool,
    loop_depth: usize,
}"""

sema_new = """pub struct SemanticAnalyzer {
    pub scopes: Vec<Scope>,
    pub functions: HashMap<String, Function>,
    pub structs: HashMap<String, StructDecl>,
    pub enums: HashMap<String, EnumDecl>,
    pub extern_fns: HashMap<String, ExternDecl>,
    pub errors: Vec<SemaError>,
    pub warnings: Vec<SemaWarning>,
    pub current_fn_return_type: Option<Type>,
    pub last_return_type: Option<Type>,
    pub type_map: HashMap<Span, Type>,
    pub resolved_names: HashMap<Span, String>,
    pub current_package: String,
    is_borrow_context: bool,
    loop_depth: usize,
}"""

content = content.replace(sema_old, sema_new)

init_old = """        let mut sa = Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            extern_fns: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            current_fn_return_type: None,
            last_return_type: None,
            type_map: HashMap::new(),
            is_borrow_context: false,
            loop_depth: 0,
        };"""

init_new = """        let mut sa = Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            extern_fns: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            current_fn_return_type: None,
            last_return_type: None,
            type_map: HashMap::new(),
            resolved_names: HashMap::new(),
            current_package: String::new(),
            is_borrow_context: false,
            loop_depth: 0,
        };"""

content = content.replace(init_old, init_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

