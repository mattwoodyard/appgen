
use std::collections::HashMap;

pub struct CreateSql(String);
pub struct RustStruct(String);

impl From<CreateSql> for String {
    fn from(c: CreateSql) -> String {
        c.0
    }
}


impl<'a> From<&'a str> for CreateSql {
    fn from(c: &str) -> CreateSql {
        CreateSql(c.into())
    }
}



impl From<String> for CreateSql {
    fn from(c: String) -> CreateSql {
        CreateSql(c)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub enum SqlOption {
    TypeOverride(String),
    Index(String),
    Unique(String),
    Constraint(String)
}



// TODO(matt) - support multivalued primary keys (@nico williams)
#[derive(Debug, Serialize, Deserialize)]
pub enum Cardinality {
    One(String, String),
    Many(String, String)
}

impl Cardinality {
    pub fn field(&self) -> &String {
        match self {
            Cardinality::One(_, f) => f,
            Cardinality::Many(_, f) => f,
        }
    }

    pub fn table(&self) -> &String {
        match self {
            Cardinality::One(f, _) => f,
            Cardinality::Many(f, _) => f,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Nullable {
    NotNull,
    Null
}

impl Default for Nullable {
    fn default() -> Nullable {
        Nullable::NotNull
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Editable {
    WriteOnce,
    ReadOnly,
    ReadWrite
}

impl Default for Editable {
    fn default() -> Editable {
        Editable::ReadWrite
    }
}






#[serde(default)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Options {
    pub null: Nullable,
    pub primary_key: bool,
    pub label: String,
    #[serde(rename = "match")]
    pub default_value: String,
    pub editable: Editable,
    /* sql: Option<SqlOptions>, */
    /* uiview: Option<UiViewOptions>, */
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MappedFieldType {
    BigSerialPk,
    Boolean,
    String(usize),
    Integer,
    Numeric,
    Timestamp(Option<String>)
}

impl MappedFieldType {
    // We lose some type safety here :(
    // This is the 'heart' of the whole thing
    pub fn view_map_type(&self, v: &ViewKind, o: &Options) -> WidgetDefinition {
        let editting = match (&o.editable, v) {
            (Editable::ReadOnly, _) => false,
            (Editable::WriteOnce, ViewKind::Create) => true,
            (Editable::WriteOnce, _) => false,
            (Editable::ReadWrite, ViewKind::Create) => true, 
            (Editable::ReadWrite, ViewKind::Edit) => true, 
            (Editable::ReadWrite, ViewKind::Show) => false, 
            (Editable::ReadWrite, ViewKind::List) => false, 
            (Editable::ReadWrite, ViewKind::Filter) => true, 
            (Editable::ReadWrite, ViewKind::Delete) => false
        };

        // Compute rw state from view and .editable
        match (self, editting) {
            (MappedFieldType::BigSerialPk, true) => WidgetDefinition::simple("TextInput"),
            (MappedFieldType::BigSerialPk, false ) => WidgetDefinition::simple("TextInput"),
            (MappedFieldType::Boolean, true) => WidgetDefinition::simple("BooleanInput"),
            (MappedFieldType::Boolean, false) => WidgetDefinition::simple("BooleanField"),
            (MappedFieldType::String(sz), true) => {
                if *sz < 128 { WidgetDefinition::simple("TextInput") } else { WidgetDefinition::simple("LongTextInput") }
            }
            (MappedFieldType::String(sz), false) => WidgetDefinition::simple("TextField"),
            (MappedFieldType::Integer, false) => WidgetDefinition::simple("NumberField"),
            (MappedFieldType::Integer, true) => WidgetDefinition::simple("NumberInput"),
            (MappedFieldType::Numeric, false) => WidgetDefinition::simple("NumberField"),
            (MappedFieldType::Numeric, true) => WidgetDefinition::simple("NumberInput"),
            (MappedFieldType::Timestamp(_), false) => WidgetDefinition::simple("TextField"),
            (MappedFieldType::Timestamp(_), true) => WidgetDefinition::simple("TextInput"),
        }
    }

}

// This should be extracted
#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetDefinition {
    name: String,
    pub source: String,
    attrs: HashMap<String, String>
}


impl WidgetDefinition {
    pub fn tag(&self) -> &String { &self.name }

    pub fn simple(n: &str) -> WidgetDefinition {
        WidgetDefinition {
            name: n.into(),
            source: "react-admin".into(),
            attrs: HashMap::new()
        }
    }
}


/* #[derive(Debug, Serialize, Deserialize)] */
/* pub struct Related {table: String, field: String, cardinality: Cardinality} */


#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub from: Cardinality,
    pub to: Cardinality,
}


#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ViewKind {
    Create,
    Edit,
    Show,
    List,
    Filter,
    Delete,
}

impl Default for ViewKind { 
    fn default() -> ViewKind {
        ViewKind::List
    }
}

impl ViewKind {

    pub fn full_view_name(&self, resource: &str) -> String {
        let x = match self {
            ViewKind::Create => { "Create" }
            ViewKind::Edit => { "Edit" }
            ViewKind::Show => { "Show" }
            ViewKind::List => { "List" }
            ViewKind::Filter => { "Filter" }
            ViewKind::Delete => { "Delete" }
        };

        format!("{}{}", capitalize(resource), x)
    }
}

fn capitalize(word: &str) -> String {
    word.chars().enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_lowercase().to_string() })
        .collect()
}

impl ViewKind {
    fn view_name(&self, resource: String) -> String {
        let rname = capitalize(&resource);
        match self {
            ViewKind::List => { rname+"List" }
            ViewKind::Edit => { rname+"Edit" }
            ViewKind::Show => { rname+"Show" }
            ViewKind::Create => { rname+"Create" }
            _ => { "".into() }
        }
    }

    pub fn view_attr_name(&self) -> String {
        match self {
            ViewKind::Create => { "create".into() }
            ViewKind::Delete => { "delete".into() }
            ViewKind::Edit => { "edit".into() }
            ViewKind::Filter => { "filter".into() }
            ViewKind::List => { "list".into() }
            ViewKind::Show => { "show".into() } 
        } 
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetSpec {
    field_name: String,
    override_type: Option<String>,
    override_label: Option<String>
}



#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UiViewSource {
    pub name: String,
    pub override_query: Option<String> 
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViewLayout {
    Tabbed(Vec<(String, Vec<String>)>),
    Flat(Vec<String>),
    Default
}

impl Default for ViewLayout {
    fn default() -> ViewLayout {
        ViewLayout::Default
    }
}

#[serde(default)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ActionSpec {
   pub name: String,
   pub params: HashMap<String, String>
}

#[serde(default)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ViewSpec {
    pub source: UiViewSource,
    pub view: ViewKind,
    // pub field_order: ViewLayout,
    pub field_order: Vec<String>,
    pub widget_override: Vec<WidgetSpec>,
    pub actions: Vec<ActionSpec>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Field(pub String, pub MappedFieldType, pub Options);


#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMap ( Vec<TypeMapEntry> );

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMapQuery {
    table: String,
    field: String,
    map_type: String, //MappedTypeField
    language: String,
    view: Option<ViewKind>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMapMatch {
    table: Option<String>,
    field: Option<String>,
    map_type: Option<String>,
    language: Option<String>,
    view: Option<ViewKind> 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMapEntry {
    #[serde(rename = "match")]
    match_on: TypeMapMatch,
    replacement: String // TODO(matt) 
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaSpec {
    pub tables: Vec<(String, Vec<Field>)>,
    pub relationships: Vec<Relation>,
    pub views: Vec<ViewSpec>,
    pub api: Vec<String>,
    pub acl: Vec<String>,
}






















