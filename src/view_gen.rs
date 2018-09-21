
use std::collections::HashMap;

use schema_spec::*;


macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $val); )*
            map
    }}
}

#[derive(Debug)]
pub enum IKind {
    Defaulted(String),
    Named(String),
}

#[derive(Debug)]
pub struct Import {
    source: String,
    symbols: Vec<IKind>,
}

impl Import {
    pub fn to_string(&self) -> String {
        let d2 = self.symbols.iter().filter_map(|i| {
            match i {
                IKind::Defaulted(s) => Some(s),
                IKind::Named(s) => None
            }
        })
        .cloned()
        .collect::<Vec<String>>();

        let nd = self.symbols.iter().filter_map(|i| {
            match i {
                IKind::Defaulted(s) => None,
                IKind::Named(s) => Some(s)
            }
        })
        .cloned()
        .collect::<Vec<String>>()
        .join(", ");

      
        let mut fullimport = vec!();
        if d2.len() > 0 {
            fullimport.extend(d2);
        }
        
        if nd.len() > 0 {
            fullimport.push(format!("{{{}}}", nd));
        }


        format!("import {} from '{}';", fullimport.join(","), self.source)
    }
    

    fn merge(&mut self, imports: Vec<Import>) -> Vec<Import> {
        imports
    }

    pub fn new(source: &str, symbols: Vec<&str>) -> Import {
       Import { source: source.into(),
                symbols: symbols.into_iter().map(|i| IKind::Named(String::from(i))).collect() }
    }

    pub fn react() -> Import {
        Import { source: "react".into(), symbols: vec!(IKind::Defaulted("React".into())) }
    }

    pub fn react_admin(v: Vec<String>) -> Import {
        Import { source: "react-admin".into(), symbols: v.into_iter().map(IKind::Named).collect() }
    }

}

impl From<WidgetDefinition> for Import {
    fn from(w: WidgetDefinition) -> Import {
        Import { source: w.source.clone(), symbols: vec!(IKind::Named(w.tag().clone()))}
    }
}



#[derive(Debug)]
pub enum JsxAttr {
    JsxString(String),
    JsxVariableRef(String),
    Object(HashMap<String, String>),
    JsxObject(HashMap<String, Box<JsxAttr>>),
    JsxRaw(String),
    Bool(bool),
    Ellipsis(String),
}

impl JsxAttr {



    fn to_string(&self, k: &str) -> String {
        match self {
            JsxAttr::Bool(tf) if *tf => { 
                format!("{}", k)
            }
            JsxAttr::Ellipsis(p) => {
                format!("{{...{}}}", p)
            }

            JsxAttr::JsxRaw(s) => {
                format!("{}={}", k, s)
            }
            JsxAttr::JsxString(s) => {
                format!("{}=\"{}\"", k, s)
            }
            JsxAttr::JsxVariableRef(r) => {
                format!("{}={{{}}}", k, r)
            }
            JsxAttr::Object(hm) => {
                let im = hm.iter().map(|(k,v)| {
                    format!("{}: {}", k, v)
                }).collect::<Vec<String>>().join(", ");
                format!("{{{}}}", im)
            }
            JsxAttr::JsxObject(hm) => {
                let im = hm.iter().map(|(k,v)| {
                    v.to_string(k)
                }).collect::<Vec<String>>().join(", ");
                format!("{{{}}}", im)
            }
            _ => { "".into() }

        }
    }
}


#[derive(Debug)]
pub enum JsxNode {
    Node(String, Option<HashMap<String, JsxAttr>>, Vec<Box<JsxNode>>),
    Leaf(String, Option<HashMap<String, JsxAttr>>),
    Text(String)
}



impl JsxNode {
    pub fn node(name: &str, attr: Option<HashMap<String, JsxAttr>>, children: Vec<JsxNode>) -> JsxNode {
        JsxNode::Node(name.into(), attr, children.into_iter().map(Box::new).collect())
    }

    fn tag(&self) -> Option<String> {
        match self {
            JsxNode::Node(tag, _, _ ) |
            JsxNode::Leaf(tag, _) => {
                Some(tag.clone())
            }

            _ => { None }
        }

    }

    fn attr_string(&self) -> String {
        match self {
            JsxNode::Node(_, Some(attrs), _) |
            JsxNode::Leaf(_, Some(attrs)) => {
                attrs.iter().map(|(k, v)| { v.to_string(k) }).collect::<Vec<String>>().join(" ")
            }
            _ => "".into()
        }
    }

    fn child_string(&self, prefix: String) -> String {
        match self {
            JsxNode::Node(_, _, kids) => {
                kids.iter().map(|k| {
                    format!("{}{}", prefix, k.to_string(false, prefix.clone() + "  "))
                }).collect::<Vec<String>>().join("\n")
            }
            _ => { "".into() }
        }
    }


    pub fn to_string(&self, top_level: bool, prefix: String) -> String {
        let attr_string = self.attr_string();
        let child_string = self.child_string(prefix);
        let tag = self.tag();

        match (tag, attr_string.len() != 0, child_string.len() != 0) {
            (Some(tag), true, true) => {
                format!("<{} {}>\n{}\n</{}>", tag, attr_string, child_string, tag)
            }

            (Some(tag), true, false) => {
                format!("<{} {}/>", tag, attr_string)
            }

            (Some(tag), false, true) => {
                format!("<{}>\n{}\n</{}>", tag, child_string, tag)
            }

            (Some(tag), false, false) => {
                format!("<{}/>", tag)
            }

            (None, true, false) => {
                child_string.clone() 
            }
            _ => { String::from("") }
        }
    }
}

pub struct JsxNodes;


impl JsxNodes {
    fn simple_form(name: &str, children: Vec<JsxNode>) -> (JsxNode, Import) {
        (JsxNode::node(name.clone(), 
            Some(hashmap!("".into() => JsxAttr::Ellipsis("props".into()))),
            vec!(JsxNode::node("SimpleForm", None, children))),
         Import::react_admin(vec!("SimpleForm".into(), name.clone().into())))
    }

    fn datagrid(name: &str, children: Vec<JsxNode>) -> (JsxNode, Import) {
        (JsxNode::node(name.clone(),
            Some(hashmap!("".into() => JsxAttr::Ellipsis("props".into()))),
            vec!(JsxNode::node("Datagrid", None, children))),
         Import::react_admin(vec!("Datagrid".into(), name.clone().into())))
    }
}


pub fn field_as_jsx(table: String, field: &Field, view: &ViewSpec) -> Option<(JsxNode, Import)> {
    //TODO(matt) - visibilty
    let widget = field.1.view_map_type(&view.view, &field.2);
    let mut hm = HashMap::new();
    hm.insert("source".into(), JsxAttr::JsxString(field.0.clone()));
    if field.2.label.len() > 0 {
        hm.insert("label".into(), JsxAttr::JsxString(field.2.label.clone()));
    }

    Some((JsxNode::node(widget.tag(), Some(hm), vec!()), widget.into()))
}



pub fn top_level_view_node(tables: &Vec<(String, Vec<Field>)>, view: &ViewSpec) -> (String, JsxNode, Vec<Import>) {
    let tt = tables.iter().find(|i| i.0 == view.source.name);
    
    if let Some(target_table) = tt {

        let mut cmap = target_table.1.iter()
            .filter_map(|i| field_as_jsx(target_table.0.clone(), &i, view).map(|o| { (i.0.clone(), o)}))
            .collect::<HashMap<String, (JsxNode, Import)>>();

        let (mut children, mut imports): (Vec<JsxNode>, Vec<Import>) =
            if view.field_order.len() > 0 {
                view.field_order.iter()
                    .filter_map(|i| { cmap.remove(i) })
                    .unzip()
            } else {
                cmap.drain().map(|(_, v)| v).unzip()
            };

        let actions = view.actions.iter().map(|a| {
            //TODO(matt) - params
            let n = a.name.clone() + "Button";
            JsxNode::node(&n, Some(HashMap::new()), vec!())
        });

        let aimports = view.actions.iter().map(|a| {
            //TODO(matt) - params
            let n = a.name.clone() + "Button";
            Import::react_admin(vec!(n.clone()))
        });

        // let (a1, a2) = actions.unzip();
        children.extend(actions);
        imports.extend(aimports);

        imports.insert(0, Import::react());

        //TODO(matt) - wrap i in option
        let (n, i) = match view.view {
            ViewKind::Create => { 
                JsxNodes::simple_form("Create", children)
            } 
            ViewKind::Edit => {
                JsxNodes::simple_form("Edit", children)
            }
            ViewKind::Show => {
                JsxNodes::simple_form("Show", children)
            }
            ViewKind::Filter => {
                JsxNodes::simple_form("Filter", children)
            }
            ViewKind::List => {
                JsxNodes::datagrid("List", children)
            }
            ViewKind::Delete => {
                (JsxNode::Text("No Node".into()), Import { source: "Bogus".into(), symbols: vec!()})
            }
        };

        imports.push(i);
        (view.view.full_view_name(&target_table.0), n, imports)
    } else {
        ("".into(), JsxNode::Text("No Node (No Table)".into()), vec!())

    }
}




